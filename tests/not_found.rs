//! 경계 1 테스트: 404 안내 페이지. 실제 라우터를 `oneshot`으로 호출한다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use short_url::api::{AppState, api_router};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn 없는_번호와_비정규형_숫자_경로는_404_안내_HTML을_응답한다(
    pool: PgPool,
) {
    let app = api_router(AppState::new(pool, "https://short.test".into()));

    for path in ["/1", "/0", "/0123", "/99999999999999999999"] {
        let res = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{path}");
        let content_type = res.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(
            content_type.starts_with("text/html"),
            "{path}: {content_type}"
        );
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("<html"), "{path}");
        assert!(
            html.contains(r#"href="/""#),
            "{path}: 생성 화면 링크가 없다"
        );
    }
}
