//! 링크 저장소. 링크는 불변이므로 공개 인터페이스는 `insert`와 `find`뿐이다.

use sqlx::PgPool;

/// 저장된 링크.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Link {
    /// 단축 번호 (DB identity가 발급)
    pub number: i64,
    /// 원본 URL
    pub original_url: String,
}

/// 링크를 저장하고 발급된 단축 번호가 담긴 링크를 돌려준다.
pub async fn insert(pool: &PgPool, original_url: &str) -> Result<Link, sqlx::Error> {
    sqlx::query_as("INSERT INTO links (original_url) VALUES ($1) RETURNING number, original_url")
        .bind(original_url)
        .fetch_one(pool)
        .await
}

/// 단축 번호로 링크를 찾는다.
pub async fn find(pool: &PgPool, number: i64) -> Result<Option<Link>, sqlx::Error> {
    sqlx::query_as("SELECT number, original_url FROM links WHERE number = $1")
        .bind(number)
        .fetch_optional(pool)
        .await
}
