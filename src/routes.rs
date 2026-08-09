use crate::{articles::Article, templates};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub article: Article,
}

pub async fn home() -> Html<String> {
    templates::page(
        "Rust Site Baseline",
        "A server-rendered proof of concept.",
        true,
        templates::home(),
    )
}

pub async fn work() -> Html<String> {
    templates::page(
        "Work",
        "Selected products, developer tools, and automation by and-rs.",
        false,
        templates::work(),
    )
}

pub async fn articles(State(state): State<Arc<AppState>>) -> Html<String> {
    templates::page(
        "Articles",
        "Markdown rendered by Rust.",
        false,
        templates::articles(&state.article),
    )
}

pub async fn article_page(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    if slug != state.article.slug {
        return not_found().await;
    }

    templates::page(
        &state.article.title,
        &state.article.description,
        false,
        templates::article(&state.article),
    )
    .into_response()
}

pub async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        templates::page(
            "Not found",
            "The requested page does not exist.",
            false,
            templates::not_found(),
        ),
    )
        .into_response()
}
