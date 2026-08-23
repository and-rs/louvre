use crate::templates;
use axum::response::{Html, IntoResponse, Response};

pub async fn home() -> Html<String> {
    templates::page(
        "Sanarte",
        "A server-rendered site baseline.",
        true,
        templates::home(),
    )
}

pub async fn not_found() -> Response {
    (
        axum::http::StatusCode::NOT_FOUND,
        templates::page(
            "Not found",
            "The requested page does not exist.",
            false,
            templates::not_found(),
        ),
    )
        .into_response()
}
