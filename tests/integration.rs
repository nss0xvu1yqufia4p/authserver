// tests/integration.rs

use actix_web::{test, http, App};
use authserver::{configure, AuthService}; // ← remove method import
use base64::{engine::general_purpose, Engine as _};

fn make_test_service() -> AuthService {
    AuthService::new(b"supersecret".to_vec(), "example.com".into()).with_seed(42)
}

#[actix_rt::test]
async fn test_cookie_roundtrip() {
    let svc = make_test_service();
    let cookie = svc.generate_cookie();
    assert!(svc.validate_cookie(&cookie).unwrap());
}

#[actix_rt::test]
async fn test_index_and_auth_endpoints() {
    let svc = make_test_service();

    let app = test::init_service(
        App::new().configure(|c| configure(c, svc.clone()))
    ).await;

    let token = svc.generate_cookie();

    let req = test::TestRequest::get()
        .uri(&format!("/{}", token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::FOUND);

    let req = test::TestRequest::get()
        .uri("/")
        .insert_header((http::header::COOKIE, format!("auth_token={token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_invalid_cookie_rejected() {
    let svc = make_test_service();

    let app = test::init_service(
        App::new().configure(|c| configure(c, svc.clone()))
    ).await;

    let bad_cookie = general_purpose::STANDARD.encode(b"short");
    let req = test::TestRequest::get()
        .uri("/")
        .insert_header((http::header::COOKIE, format!("auth_token={bad_cookie}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
}

