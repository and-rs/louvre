use crate::templates::{self, PageMetadata};
use axum::{
    Form,
    extract::{Path, State},
    http::{
        HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Redirect, Response},
};
use axum_login::AuthSession;
use louvre_auth::{Credentials, StudioBackend};
use louvre_storage::{PgPool, Storage, StorageError};
use std::sync::Arc;

pub struct AppState {
    pub storage: Storage,
    pub database: PgPool,
    pub studio_enabled: bool,
}

pub async fn studio_login(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<StudioBackend>,
) -> Response {
    if !state.studio_enabled {
        return studio_login_unavailable();
    }
    if auth_session.user.is_some() {
        return no_store(Redirect::to("/studio").into_response());
    }

    render_studio_login(None)
}

pub async fn studio_login_submit(
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<StudioBackend>,
    Form(credentials): Form<Credentials>,
) -> Response {
    if !state.studio_enabled {
        return studio_login_unavailable();
    }

    match auth_session.authenticate(credentials).await {
        Ok(Some(user)) => match auth_session.login(&user).await {
            Ok(()) => no_store(Redirect::to("/studio").into_response()),
            Err(error) => {
                tracing::error!(%error, "failed to create studio session");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => no_store(
            (
                StatusCode::UNAUTHORIZED,
                templates::page(
                    PageMetadata {
                        page_title: Some("Studio sign in"),
                        description: "Sign in to the private artwork workspace.",
                    },
                    templates::studio_login(Some("Those details did not match."), false),
                ),
            )
                .into_response(),
        ),
        Err(error) => {
            tracing::error!(%error, "studio authentication failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio(auth_session: AuthSession<StudioBackend>) -> Response {
    let Some(user) = auth_session.user else {
        return Redirect::to("/studio/login").into_response();
    };

    no_store(
        templates::page(
            PageMetadata {
                page_title: Some("Studio"),
                description: "Private artwork management workspace.",
            },
            templates::studio(&user.username),
        )
        .into_response(),
    )
}

pub async fn studio_logout(mut auth_session: AuthSession<StudioBackend>) -> Response {
    match auth_session.logout().await {
        Ok(_) => no_store(Redirect::to("/studio/login").into_response()),
        Err(error) => {
            tracing::error!(%error, "failed to end studio session");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn render_studio_login(error: Option<&str>) -> Response {
    no_store(
        templates::page(
            PageMetadata {
                page_title: Some("Studio sign in"),
                description: "Sign in to the private artwork workspace.",
            },
            templates::studio_login(error, false),
        )
        .into_response(),
    )
}

fn studio_login_unavailable() -> Response {
    no_store(
        (
            StatusCode::SERVICE_UNAVAILABLE,
            templates::page(
                PageMetadata {
                    page_title: Some("Studio unavailable"),
                    description: "The private artwork workspace is not configured.",
                },
                templates::studio_login(None, true),
            ),
        )
            .into_response(),
    )
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    response
}

pub async fn home() -> Response {
    templates::page(
        PageMetadata {
            page_title: None,
            description: "An artwork publishing platform.",
        },
        templates::home(),
    )
    .into_response()
}

pub async fn health(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.database.acquire().await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            tracing::warn!(%error, "failed to get PostgreSQL connection for health check");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

pub async fn artwork(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let valid = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
    };
    if !valid(&id) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match state.storage.list(&format!("artworks/{id}/")).await {
        Ok(files) if !files.is_empty() => templates::page(
            PageMetadata {
                page_title: Some(&id),
                description: "Artwork",
            },
            templates::artwork(&id, &files),
        )
        .into_response(),
        Ok(_) => not_found().await,
        Err(error) => {
            tracing::warn!(%error, artwork_id = %id, "failed to list artwork images");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn artwork_image(
    State(state): State<Arc<AppState>>,
    Path((id, file)): Path<(String, String)>,
) -> Response {
    let valid = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
    };
    if !valid(&id) || !valid(&file) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let content_type = match file.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    match state.storage.get(&format!("artworks/{id}/{file}")).await {
        Ok(bytes) => (
            [
                (CONTENT_TYPE, content_type),
                (CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Err(StorageError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(StorageError::Other(error)) => {
            tracing::warn!(%error, key = %format!("artworks/{id}/{file}"), "failed to fetch artwork image");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        templates::page(
            PageMetadata {
                page_title: Some("Not found"),
                description: "The requested page does not exist.",
            },
            templates::not_found(),
        ),
    )
        .into_response()
}
