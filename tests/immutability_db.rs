//! 경계 2 테스트: DB에 SQL을 직접 실행해도 링크를 바꾸거나 지울 수 없다 (ADR-0002의 세 번째 방어선).
#![cfg(feature = "server")]
#![allow(non_snake_case)]

use short_url::links;
use sqlx::PgPool;

const ORIGINAL: &str = "https://example.com/original";

/// 링크 하나를 저장하고 단축 번호를 돌려준다.
async fn seed(pool: &PgPool) -> i64 {
    links::insert(pool, ORIGINAL).await.unwrap().number
}

/// 링크가 처음 값 그대로 남아 있는지 확인한다.
async fn assert_unchanged(pool: &PgPool, number: i64) {
    let link = links::find(pool, number)
        .await
        .unwrap()
        .expect("링크가 남아 있어야 한다");
    assert_eq!(link.original_url, ORIGINAL);
}

#[sqlx::test]
async fn UPDATE는_거부된다(pool: PgPool) {
    let number = seed(&pool).await;
    let result =
        sqlx::query("UPDATE links SET original_url = 'https://evil.example/' WHERE number = $1")
            .bind(number)
            .execute(&pool)
            .await;
    assert!(result.is_err(), "UPDATE가 성공하면 안 된다");
    assert_unchanged(&pool, number).await;
}

#[sqlx::test]
async fn DELETE는_거부된다(pool: PgPool) {
    let number = seed(&pool).await;
    let result = sqlx::query("DELETE FROM links WHERE number = $1")
        .bind(number)
        .execute(&pool)
        .await;
    assert!(result.is_err(), "DELETE가 성공하면 안 된다");
    assert_unchanged(&pool, number).await;
}

#[sqlx::test]
async fn TRUNCATE는_거부된다(pool: PgPool) {
    let number = seed(&pool).await;
    let result = sqlx::query("TRUNCATE links").execute(&pool).await;
    assert!(result.is_err(), "TRUNCATE가 성공하면 안 된다");
    assert_unchanged(&pool, number).await;
}

#[sqlx::test]
async fn 번호를_지정한_INSERT는_거부된다(pool: PgPool) {
    let number = seed(&pool).await;
    let result = sqlx::query(
        "INSERT INTO links (number, original_url) VALUES ($1, 'https://evil.example/')",
    )
    .bind(number + 1)
    .execute(&pool)
    .await;
    assert!(result.is_err(), "번호를 지정한 INSERT가 성공하면 안 된다");
    assert!(links::find(&pool, number + 1).await.unwrap().is_none());
    assert_unchanged(&pool, number).await;
}
