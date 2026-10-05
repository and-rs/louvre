use axum::{
    Form, Router,
    http::{Request, StatusCode, header},
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, AuthSession, login_required};
use louvre_auth::{Credentials, StudioBackend};
use sqlx::PgPool;
use tower::ServiceExt;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

async fn login(
    mut auth_session: AuthSession<StudioBackend>,
    Form(credentials): Form<Credentials>,
) -> StatusCode {
    match auth_session.authenticate(credentials).await.unwrap() {
        Some(user) => {
            auth_session.login(&user).await.unwrap();
            StatusCode::OK
        }
        None => StatusCode::UNAUTHORIZED,
    }
}

async fn studio(auth_session: AuthSession<StudioBackend>) -> StatusCode {
    if auth_session.user.is_some() {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}

async fn logout(mut auth_session: AuthSession<StudioBackend>) -> StatusCode {
    match auth_session.logout().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn app(pool: PgPool, backend: StudioBackend) -> Router {
    let session_layer = SessionManagerLayer::new(PostgresStore::new(pool))
        .with_name("louvre_studio_session")
        .with_path("/studio")
        .with_secure(false);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    Router::new()
        .route("/studio", get(studio))
        .route("/studio/logout", post(logout))
        .route_layer(login_required!(StudioBackend, login_url = "/studio/login"))
        .route("/studio/login", post(login))
        .layer(auth_layer)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to an isolated PostgreSQL database"]
async fn a_session_created_by_one_instance_is_shared_and_revocable_by_another() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("set TEST_DATABASE_URL to run this PostgreSQL integration test");
    let pool_one = PgPool::connect(&database_url).await.unwrap();
    let pool_two = PgPool::connect(&database_url).await.unwrap();
    PostgresStore::new(pool_one.clone())
        .migrate()
        .await
        .unwrap();

    let password_hash = louvre_auth::generate_password_hash("correct horse");
    let backend_one = StudioBackend::from_password_hash("owner", &password_hash);
    let backend_two = StudioBackend::from_password_hash("owner", password_hash);
    let instance_one = app(pool_one, backend_one);
    let instance_two = app(pool_two, backend_two);

    let login_response = instance_one
        .clone()
        .oneshot(
            Request::post("/studio/login")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(axum::body::Body::from(
                    "username=owner&password=correct+horse",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    let on_second_instance = instance_two
        .clone()
        .oneshot(
            Request::get("/studio")
                .header(header::COOKIE, &cookie)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(on_second_instance.status(), StatusCode::OK);

    let logout_response = instance_two
        .oneshot(
            Request::post("/studio/logout")
                .header(header::COOKIE, &cookie)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_response.status(), StatusCode::OK);

    let after_logout = instance_one
        .oneshot(
            Request::get("/studio")
                .header(header::COOKIE, cookie)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(after_logout.status(), StatusCode::TEMPORARY_REDIRECT);
}
