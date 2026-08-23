mod assets;
mod routes;
mod storage;
mod templates;
use axum::{Router, http::StatusCode, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{routes::AppState, storage::Storage};

#[cfg(feature = "dev")]
use axum::http::{HeaderValue, header::CACHE_CONTROL};
#[cfg(feature = "dev")]
use tower::ServiceBuilder;
#[cfg(feature = "dev")]
use tower_http::set_header::SetResponseHeaderLayer;
#[cfg(feature = "dev")]
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let static_files = ServeDir::new("src/static");
    #[cfg(feature = "dev")]
    let static_files = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .service(static_files);
    #[cfg(not(feature = "dev"))]
    let static_files = static_files.precompressed_br();

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "sanarte-artworks".to_string());
    let state = Arc::new(AppState {
        storage: Storage::new(s3_client, bucket),
    });

    let app = Router::new()
        .route("/", get(routes::home))
        .route("/artwork/{id}", get(routes::artwork))
        .route("/artwork/{id}/image/{file}", get(routes::artwork_image))
        .route("/health", get(|| async { StatusCode::OK }))
        .fallback(routes::not_found)
        .nest_service("/static", static_files)
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
