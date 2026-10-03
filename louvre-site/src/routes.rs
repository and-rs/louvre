use crate::templates::{self, PageMetadata};
use axum::{
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use louvre_storage::{DbPool, Storage, StorageError};
use std::sync::Arc;

pub struct AppState {
    pub storage: Storage,
    pub database: Option<DbPool>,
}

pub async fn home(State(state): State<Arc<AppState>>) -> Response {
    let message = match &state.database {
        Some(pool) => match greeting(pool).await {
            Ok(message) => message,
            Err(status) => return status.into_response(),
        },
        None => "Hello, world!".to_owned(),
    };

    templates::page(
        PageMetadata {
            page_title: None,
            description: "An artwork publishing platform.",
        },
        templates::home(&message),
    )
    .into_response()
}

pub async fn database_hello(State(state): State<Arc<AppState>>) -> Response {
    let Some(pool) = &state.database else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "DATABASE_URL is not configured",
        )
            .into_response();
    };

    match greeting(pool).await {
        Ok(message) => message.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn greeting(pool: &DbPool) -> Result<String, StatusCode> {
    let mut connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => {
            tracing::warn!(%error, "failed to get PostgreSQL connection");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    match louvre_storage::hello(&mut connection).await {
        Ok(message) => Ok(message),
        Err(error) => {
            tracing::warn!(%error, "PostgreSQL hello query failed");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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
