//! 경계 1 테스트: 실제 라우터를 `oneshot`으로 호출한다. 테스트마다 격리된 DB를 쓴다.
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

#[sqlx::test]
async fn 생성한_링크의_단축_주소로_이동하면_301로_원본_URL이_온다(
    pool: PgPool,
) {
    let app = api_router(AppState::new(pool, BASE.into()));
    let original = "https://example.com/a/b?x=1&y=2#frag";

    let res = app.clone().oneshot(post_link(original)).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let number = json["number"].as_i64().unwrap();
    assert_eq!(json["short_url"], format!("{BASE}/{number}"));
    assert_eq!(json["original_url"], original);

    let res = app
        .oneshot(
            Request::get(format!("/{number}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(res.headers()[header::LOCATION], original);
}
