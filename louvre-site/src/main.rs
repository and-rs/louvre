mod assets;
mod routes;
mod templates;
use axum::{Router, routing::get, routing::post};
use axum_login::{AuthManagerLayerBuilder, login_required};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::AbortHandle;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::routes::AppState;
use louvre_auth::StudioBackend;
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
    match std::env::args().nth(1).as_deref() {
        Some("migrate") => {
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set to run database migrations");
            if let Err(error) = apply_migrations(&database_url).await {
                eprintln!("database migration failed: {error}");
                std::process::exit(1);
            }
            println!("Database migrations are up to date.");
            return;
        }
        Some(command) => {
            eprintln!("unknown command: {command}");
            std::process::exit(2);
        }
        None => {}
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let studio_backend = StudioBackend::from_env();
    let studio_enabled = studio_backend.is_configured();
    if !studio_enabled {
        tracing::info!(
            "studio is disabled; set STUDIO_USERNAME and STUDIO_PASSWORD_HASH to enable it"
        );
    }

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
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run the application");
    let database = postgres_pool(&database_url)
        .await
        .expect("failed to connect to PostgreSQL");
    let state = Arc::new(AppState {
        storage,
        database: database.clone(),
        studio_enabled,
    });

    let session_store = louvre_auth::session_store(database.clone());
    let session_cleanup = louvre_auth::spawn_session_cleanup(session_store.clone());
    let session_layer = louvre_auth::session_layer(session_store, !cfg!(feature = "dev"));
    let auth_layer = AuthManagerLayerBuilder::new(studio_backend, session_layer).build();

    let app = Router::new()
        .route("/studio", get(routes::studio))
        .route_layer(login_required!(StudioBackend, login_url = "/studio/login"))
        .route(
            "/studio/login",
            get(routes::studio_login).post(routes::studio_login_submit),
        )
        .route("/studio/logout", post(routes::studio_logout))
        .route("/", get(routes::home))
        .route("/artwork/{id}", get(routes::artwork))
        .route("/artwork/{id}/image/{file}", get(routes::artwork_image))
        .route("/health", get(routes::health))
        .fallback(routes::not_found)
        .nest_service("/static", static_files)
        .layer(TraceLayer::new_for_http())
        .layer(auth_layer)
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
        .with_graceful_shutdown(shutdown_signal(session_cleanup.abort_handle()))
        .await
        .expect("server failed");
    session_cleanup.abort();
}

async fn apply_migrations(
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database = postgres_pool(database_url).await?;
    louvre_storage::run_migrations(&database).await?;
    louvre_auth::migrate_sessions(&database).await?;
    Ok(())
}

async fn shutdown_signal(session_cleanup: AbortHandle) {
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
        _ = ctrl_c => { session_cleanup.abort(); },
        _ = terminate => { session_cleanup.abort(); },
    }

    tracing::info!("shutdown signal received");
}
