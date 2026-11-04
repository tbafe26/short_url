//! 경계 1 테스트: 동시 발급에서 단축 번호가 겹치지 않고, 서버를 다시 만들어도 기존 링크가 유지된다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use std::collections::HashSet;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use short_url::api::{AppState, api_router};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

const BASE: &str = "https://short.test";

/// 링크를 발급하고 단축 번호를 돌려준다.
async fn issue(app: Router, url: String) -> (i64, String) {
    let req = Request::post("/api/links")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::json!({ "url": url }).to_string()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (json["number"].as_i64().unwrap(), url)
}

/// 단축 번호로 이동했을 때의 `Location`을 돌려준다.
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
async fn 서로_다른_URL_50개를_동시에_발급해도_번호가_겹치지_않고_각자_원본으로_이동한다(
    pool: PgPool,
) {
    // 여러 요청이 실제로 동시에 DB에 들어가도록 연결을 넉넉히 둔 풀을 따로 만든다.
    let wide_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    let app = api_router(AppState::new(wide_pool, BASE.into()));

    let mut tasks = tokio::task::JoinSet::new();
    for i in 0..50 {
        let app = app.clone();
        tasks.spawn(issue(
            app,
            format!("https://example.com/page/{i}?q={i}#s{i}"),
        ));
    }
    let issued: Vec<(i64, String)> = tasks.join_all().await;

    let numbers: HashSet<i64> = issued.iter().map(|(n, _)| *n).collect();
    assert_eq!(numbers.len(), 50, "단축 번호가 겹치면 안 된다");

    for (number, url) in &issued {
        assert_eq!(&location_of(&app, *number).await, url);
    }
}

#[sqlx::test]
async fn 풀과_라우터를_새로_만들어도_기존_번호는_같은_원본으로_이동한다(
    pool: PgPool,
) {
    let options = (*pool.connect_options()).clone();
    let original = "https://example.com/persist?x=1#keep".to_string();

    let first = api_router(AppState::new(pool.clone(), BASE.into()));
    let (number, _) = issue(first, original.clone()).await;
    // 재시작을 흉내 낸다: 기존 풀을 닫고 같은 DB에 새로 연결한다.
    pool.close().await;

    let new_pool = PgPoolOptions::new().connect_with(options).await.unwrap();
    let restarted = api_router(AppState::new(new_pool, BASE.into()));
    assert_eq!(location_of(&restarted, number).await, original);
}

#[sqlx::test]
async fn 같은_원본_URL을_두_번_발급하면_서로_다른_번호가_나온다(pool: PgPool) {
    let app = api_router(AppState::new(pool, BASE.into()));
    let url = "https://example.com/same".to_string();

    let (first, _) = issue(app.clone(), url.clone()).await;
    let (second, _) = issue(app.clone(), url.clone()).await;

    assert_ne!(first, second);
    assert_eq!(location_of(&app, first).await, url);
    assert_eq!(location_of(&app, second).await, url);
}
