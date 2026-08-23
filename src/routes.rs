use crate::{
    storage::{Storage, StorageError},
    templates,
};
use axum::{
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

pub struct AppState {
    pub storage: Storage,
}

pub async fn home() -> Html<String> {
    templates::page(
        "Sanarte",
        "A server-rendered site baseline.",
        templates::home(),
    )
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
        Ok(files) if !files.is_empty() => {
            templates::page(&id, "Artwork", templates::artwork(&id, &files)).into_response()
        }
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
            "Not found",
            "The requested page does not exist.",
            templates::not_found(),
        ),
    )
        .into_response()
}
