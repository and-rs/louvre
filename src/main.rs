mod articles;
mod routes;
mod templates;
use axum::{Router, http::StatusCode, routing::get};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[cfg(feature = "dev")]
use tower_livereload::LiveReloadLayer;

use crate::{articles::parse_article, routes::AppState};

const ARTICLE_SOURCE: &str = include_str!("../content/articles/faster-by-default.md");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let article = parse_article("faster-by-default", ARTICLE_SOURCE)
        .expect("embedded article must have valid frontmatter");
    let state = Arc::new(AppState { article });

    let app = Router::new()
        .route("/", get(routes::home))
        .route("/articles", get(routes::articles))
        .route("/articles/{slug}", get(routes::article_page))
        .route("/health", get(|| async { StatusCode::OK }))
        .fallback(routes::not_found)
        .nest_service("/static", ServeDir::new("src/static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    #[cfg(feature = "dev")]
    let app = app.layer(LiveReloadLayer::new());

    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address)
        .await
        .expect("local port 3000 must be available");
    tracing::info!(%address, "listening");
    axum::serve(listener, app).await.expect("server failed");
}
