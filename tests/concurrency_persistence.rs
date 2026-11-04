//! 경계 1 테스트: 동시 발급에서 단축 번호가 겹치지 않고, 서버를 다시 만들어도 기존 링크가 유지된다.
#![cfg(feature = "server")]
#![allow(non_snake_case)]

mod common;

use std::collections::HashSet;

use common::{app, issue, location_of};
use sqlx::{PgPool, postgres::PgPoolOptions};

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
    let app = app(wide_pool);

    let mut tasks = tokio::task::JoinSet::new();
    for i in 0..50 {
        let app = app.clone();
        let url = format!("https://example.com/page/{i}?q={i}#s{i}");
        tasks.spawn(async move { (issue(&app, &url).await, url) });
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
    let original = "https://example.com/persist?x=1#keep";

    let number = issue(&app(pool.clone()), original).await;
    // 재시작을 흉내 낸다: 기존 풀을 닫고 같은 DB에 새로 연결한다.
    pool.close().await;

    let new_pool = PgPoolOptions::new().connect_with(options).await.unwrap();
    assert_eq!(location_of(&app(new_pool), number).await, original);
}

#[sqlx::test]
async fn 같은_원본_URL을_두_번_발급하면_서로_다른_번호가_나온다(pool: PgPool) {
    let app = app(pool);
    let url = "https://example.com/same";

    let first = issue(&app, url).await;
    let second = issue(&app, url).await;

    assert_ne!(first, second);
    assert_eq!(location_of(&app, first).await, url);
    assert_eq!(location_of(&app, second).await, url);
}
