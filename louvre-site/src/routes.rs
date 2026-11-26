use crate::templates::{self, PageMetadata};
use axum::{
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
    },
    response::{IntoResponse, Response},
};
use louvre_storage::{
    PgPool, Storage, StorageError, get_artwork, list_published_artworks, valid_slug,
};
use std::sync::Arc;

pub struct AppState {
    pub storage: Storage,
    pub database: PgPool,
}

pub async fn home(State(state): State<Arc<AppState>>) -> Response {
    let artworks = match list_published_artworks(&state.database).await {
        Ok(artworks) => artworks,
        Err(error) => {
            tracing::error!(%error, "failed to load public artwork collection");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    templates::page(
        PageMetadata {
            page_title: None,
            description: "An artwork publishing platform.",
        },
        templates::home(&artworks),
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

pub async fn artwork(State(state): State<Arc<AppState>>, Path(slug): Path<String>) -> Response {
    match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) if artwork.status == "published" => templates::page(
            PageMetadata {
                page_title: Some(&artwork.title),
                description: artwork.description.as_deref().unwrap_or(&format!(
                    "Artwork from the {} collection.",
                    crate::branding::site_name()
                )),
            },
            templates::artwork(&artwork),
        )
        .into_response(),
        Ok(_) => not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load public artwork");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn artwork_image(
    State(state): State<Arc<AppState>>,
    Path((slug, file)): Path<(String, String)>,
) -> Response {
    if !valid_slug(&slug)
        || file.is_empty()
        || !file.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.' || byte == b'_'
        })
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let artwork = match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) if artwork.status == "published" => artwork,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load artwork for image request");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(version) = artwork
        .images
        .iter()
        .flat_map(|image| image.web_versions.iter())
        .find(|version| version.s3_key.rsplit('/').next() == Some(file.as_str()))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let key = &version.s3_key;

    match state.storage.get(key).await {
        Ok(bytes) => (
            [
                (CONTENT_TYPE, version.content_type.as_str()),
                (CACHE_CONTROL, "public, max-age=31536000, immutable"),
                (X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response(),
        Err(StorageError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(StorageError::Other(error)) => {
            tracing::warn!(%error, %key, "failed to fetch artwork image");
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
