//! 경계 1 테스트의 공용 도우미. 실제 라우터를 `oneshot`으로 호출한다.
#![allow(dead_code)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    response::Response,
};
use short_url::api::{AppState, api_router};
use sqlx::PgPool;
use tower::ServiceExt;

/// 테스트에서 쓰는 서비스 공개 주소 (`PUBLIC_BASE_URL`)
pub const BASE: &str = "https://short.test";

/// 테스트 DB 풀로 라우터를 만든다.
pub fn app(pool: PgPool) -> Router {
    api_router(AppState::new(pool, BASE.into()))
}

/// 링크 생성 요청을 만든다.
pub fn post_link(url: &str) -> Request<Body> {
    Request::post("/api/links")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "url": url }).to_string()))
        .unwrap()
}

/// 응답 본문을 JSON으로 읽는다.
pub async fn json_of(res: Response) -> serde_json::Value {
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// 링크를 발급하고 단축 번호를 돌려준다.
pub async fn issue(app: &Router, url: &str) -> i64 {
    let res = app.clone().oneshot(post_link(url)).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED, "발급 실패: {url}");
    json_of(res).await["number"].as_i64().unwrap()
}

/// 단축 번호로 이동했을 때의 `Location`을 돌려준다. 301이 아니면 실패한다.
pub async fn location_of(app: &Router, number: i64) -> String {
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
