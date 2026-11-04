//! 경계 1 테스트: 링크 변경 요청(`PUT`/`PATCH`/`DELETE`)은 405로 거부되고, 기존 이동은 그대로다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use short_url::api::{AppState, api_router};
use sqlx::PgPool;
use tower::ServiceExt;

const BASE: &str = "https://short.test";

/// 링크를 하나 발급하고 단축 번호를 돌려준다.
async fn issue(app: &Router, url: &str) -> i64 {
    let req = Request::post("/api/links")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "url": url }).to_string()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&body).unwrap()["number"]
        .as_i64()
        .unwrap()
}

/// 변경 요청을 보내고 405, `Allow` 헤더, 불변성 설명 본문을 확인한다.
async fn assert_rejected(app: &Router, method: Method, number: i64) {
    let req = Request::builder()
        .method(method.clone())
        .uri(format!("/api/links/{number}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({ "url": "https://evil.example/" }).to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED, "{method}");
    assert!(res.headers().contains_key(header::ALLOW), "{method}");
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["error"].as_str().unwrap().contains("변경할 수 없"),
        "{method}"
    );
}

async fn location_of(app: &Router, number: i64) -> String {
    let req = Request::get(format!("/{number}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
    res.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_string()
}

#[sqlx::test]
async fn 기존_링크에_대한_변경_요청은_405로_거부되고_이동은_그대로다(
    pool: PgPool,
) {
    let app = api_router(AppState::new(pool, BASE.into()));
    let original = "https://example.com/keep?x=1#top";
    let number = issue(&app, original).await;

    for method in [Method::PUT, Method::PATCH, Method::DELETE] {
        assert_rejected(&app, method, number).await;
    }

    assert_eq!(location_of(&app, number).await, original);
}

#[sqlx::test]
async fn 없는_번호에_대한_변경_요청도_405로_거부된다(pool: PgPool) {
    let app = api_router(AppState::new(pool, BASE.into()));

    for method in [Method::PUT, Method::PATCH, Method::DELETE] {
        assert_rejected(&app, method, 999_999).await;
    }
}
