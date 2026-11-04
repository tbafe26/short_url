//! 경계 1 테스트: 원본 URL 검증을 라우터를 통해 확인한다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use short_url::api::{AppState, api_router};
use sqlx::PgPool;
use tower::ServiceExt;

const BASE: &str = "https://short.test";

fn post_link(url: &str) -> Request<Body> {
    Request::post("/api/links")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "url": url }).to_string()))
        .unwrap()
}

async fn json_of(res: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[sqlx::test]
async fn 잘못된_원본_URL은_400과_메시지로_거부되고_링크가_만들어지지_않는다(
    pool: PgPool,
) {
    let app = api_router(AppState::new(pool.clone(), BASE.into()));
    let too_long = format!("https://example.com/{}", "a".repeat(2048));
    let cases = [
        "ftp://example.com/file",
        "javascript:alert(1)",
        "http://",
        "",
        "   ",
        "not a url",
        "https://short.test/5",
        "https://SHORT.test/5",
        too_long.as_str(),
    ];
    for input in cases {
        let res = app.clone().oneshot(post_link(input)).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "입력: {input:?}");
        let json = json_of(res).await;
        let message = json["error"].as_str().unwrap_or("");
        assert!(!message.is_empty(), "오류 메시지가 있어야 한다: {input:?}");
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM links")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn 앞뒤_공백은_제거되고_정규화한_값이_저장되며_경로_쿼리_프래그먼트가_보존된다(
    pool: PgPool,
) {
    let app = api_router(AppState::new(pool, BASE.into()));
    let res = app
        .clone()
        .oneshot(post_link("  HTTPS://Example.COM:443/a/b?x=1&y=2#frag \n"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let json = json_of(res).await;
    let expected = "https://example.com/a/b?x=1&y=2#frag";
    assert_eq!(json["original_url"], expected);

    let number = json["number"].as_i64().unwrap();
    let res = app
        .oneshot(
            Request::get(format!("/{number}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.headers()[header::LOCATION], expected);
}
