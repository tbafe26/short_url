//! API 라우터 스모크 테스트: 프로세스 안에서 라우터를 직접 호출한다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use axum::{body::Body, http::Request, http::StatusCode};
use short_url::api::{AppState, api_router};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn health_는_200을_응답한다(pool: PgPool) {
    let app = api_router(AppState::new(pool, "https://short.test".into()));
    let res = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
