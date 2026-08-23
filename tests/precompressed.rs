use http::header::ACCEPT_ENCODING;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[tokio::test]
async fn serves_brotli_variant_when_accepted() {
    let svc = ServeDir::new("src/static").precompressed_br();
    let req = Request::builder()
        .header(ACCEPT_ENCODING, "gzip, br")
        .uri("/css/site.css")
        .body(())
        .unwrap();

    let res = svc.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("content-encoding").unwrap(), "br");
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert!(!body.is_empty());
}

#[tokio::test]
async fn serves_uncompressed_without_accept_encoding() {
    let svc = ServeDir::new("src/static").precompressed_br();
    let req = Request::builder().uri("/css/site.css").body(()).unwrap();

    let res = svc.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn serves_brotli_js_asset() {
    let svc = ServeDir::new("src/static").precompressed_br();
    let req = Request::builder()
        .header(ACCEPT_ENCODING, "br")
        .uri("/js/mu.min.js")
        .body(())
        .unwrap();

    let res = svc.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("content-encoding").unwrap(), "br");
}
