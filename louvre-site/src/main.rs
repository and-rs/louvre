mod assets;
mod branding;
mod routes;
mod templates;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::routes::AppState;
use louvre_storage::{Storage, postgres_pool};

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
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into())
                .from_env_lossy(),
        )
        .init();

    let static_files = ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/static"));
    #[cfg(feature = "dev")]
    let static_files = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .service(static_files);
    #[cfg(not(feature = "dev"))]
    let static_files = static_files.precompressed_br();

    let storage = Storage::from_env().await;
    #[cfg(feature = "dev")]
    let storage = storage.with_local_uploads(
        std::env::var("DEV_UPLOADS_DIR").unwrap_or_else(|_| ".local/uploads".into()),
    );
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run the application");
    #[cfg(feature = "dev")]
    {
        let options: sqlx::postgres::PgConnectOptions = database_url
            .parse()
            .expect("invalid development DATABASE_URL");
        assert!(
            matches!(options.get_host(), "localhost" | "127.0.0.1" | "::1"),
            "development requires local PostgreSQL; refusing remote DATABASE_URL"
        );
    }
    let database = postgres_pool(&database_url)
        .await
        .expect("failed to connect to PostgreSQL");
    let state = Arc::new(AppState { storage, database });

    let app = Router::new()
        .route("/", get(routes::home))
        .route("/artwork/{id}", get(routes::artwork))
        .route("/artwork/{id}/image/{file}", get(routes::artwork_image))
        .route("/health", get(routes::health))
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
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
