//! 경계 1 테스트: 원본 URL 검증을 라우터를 통해 확인한다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{app, issue, json_of, location_of, post_link};
use sqlx::PgPool;
use tower::ServiceExt;

/// 링크가 하나도 만들어지지 않았는지 확인한다.
async fn assert_no_links(pool: &PgPool) {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM links")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn 잘못된_원본_URL은_400과_메시지로_거부되고_링크가_만들어지지_않는다(
    pool: PgPool,
) {
    let app = app(pool.clone());
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
    assert_no_links(&pool).await;
}

#[sqlx::test]
async fn 형식이_잘못된_요청_본문도_400과_메시지로_거부된다(pool: PgPool) {
    let app = app(pool.clone());
    let cases: [(Option<&str>, &str); 4] = [
        (Some("application/json"), "{not json"),
        (
            Some("application/json"),
            r#"{"link":"https://example.com"}"#,
        ),
        (Some("application/json"), r#"{"url":123}"#),
        (None, r#"{"url":"https://example.com"}"#),
    ];
    for (content_type, body) in cases {
        let mut req = Request::post("/api/links");
        if let Some(ct) = content_type {
            req = req.header(header::CONTENT_TYPE, ct);
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST, "본문: {body}");
        let json = json_of(res).await;
        assert!(
            !json["error"].as_str().unwrap_or("").is_empty(),
            "오류 메시지가 있어야 한다: {body}"
        );
    }
    assert_no_links(&pool).await;
}

#[sqlx::test]
async fn 앞뒤_공백은_제거되고_정규화한_값이_저장되며_경로_쿼리_프래그먼트가_보존된다(
    pool: PgPool,
) {
    let app = app(pool);
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
    assert_eq!(location_of(&app, number).await, expected);
    // 같은 원본 URL을 다시 넣어도 새 번호가 나온다 (정규화 후 같은 값이어도 마찬가지).
    assert_ne!(issue(&app, expected).await, number);
}
