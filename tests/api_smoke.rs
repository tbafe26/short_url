//! API 라우터 스모크 테스트: 프로세스 안에서 라우터를 직접 호출한다.
#![cfg(feature = "server")]

use axum::{body::Body, http::Request, http::StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn health_는_200을_응답한다() {
    let app = short_url::api::api_router();
    let res = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
