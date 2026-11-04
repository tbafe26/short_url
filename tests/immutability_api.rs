//! 경계 1 테스트: 링크 변경 요청(`PUT`/`PATCH`/`DELETE`)은 405로 거부되고, 기존 이동은 그대로다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use common::{app, issue, json_of, location_of};
use sqlx::PgPool;
use tower::ServiceExt;

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
    let json = json_of(res).await;
    assert!(
        json["error"].as_str().unwrap().contains("변경할 수 없"),
        "{method}"
    );
}

#[sqlx::test]
async fn 기존_링크에_대한_변경_요청은_405로_거부되고_이동은_그대로다(
    pool: PgPool,
) {
    let app = app(pool);
    let original = "https://example.com/keep?x=1#top";
    let number = issue(&app, original).await;

    for method in [Method::PUT, Method::PATCH, Method::DELETE] {
        assert_rejected(&app, method, number).await;
    }

    assert_eq!(location_of(&app, number).await, original);
}

#[sqlx::test]
async fn 없는_번호에_대한_변경_요청도_405로_거부된다(pool: PgPool) {
    let app = app(pool);

    for method in [Method::PUT, Method::PATCH, Method::DELETE] {
        assert_rejected(&app, method, 999_999).await;
    }
}
