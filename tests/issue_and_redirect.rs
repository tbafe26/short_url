//! 경계 1 테스트: 실제 라우터를 `oneshot`으로 호출한다. 테스트마다 격리된 DB를 쓴다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

mod common;

use axum::http::StatusCode;
use common::{BASE, app, json_of, location_of, post_link};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn 생성한_링크의_단축_주소로_이동하면_301로_원본_URL이_온다(
    pool: PgPool,
) {
    let app = app(pool);
    let original = "https://example.com/a/b?x=1&y=2#frag";

    let res = app.clone().oneshot(post_link(original)).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let json = json_of(res).await;
    let number = json["number"].as_i64().unwrap();
    assert_eq!(json["short_url"], format!("{BASE}/{number}"));
    assert_eq!(json["original_url"], original);

    assert_eq!(location_of(&app, number).await, original);
}
