mod assets;
mod routes;
mod templates;
use axum::{Router, http::StatusCode, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};

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

    let app = Router::new()
        .route("/", get(routes::home))
        .route("/health", get(|| async { StatusCode::OK }))
        .fallback(routes::not_found)
        .nest_service("/static", static_files)
        .layer(TraceLayer::new_for_http());

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
