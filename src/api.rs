//! 서버 API 라우터. Dioxus 서버에 합쳐 쓰고, 테스트에서는 단독으로 호출한다.

use axum::{Router, routing::get};

/// 단축 서비스의 API 라우터를 만든다.
///
/// 주의: Dioxus 라우터가 이미 fallback을 가지므로 여기서 `.fallback()`을 달지 않는다.
pub fn api_router() -> Router {
    Router::new().route("/api/health", get(health))
}

/// 배포 환경의 헬스체크용. 항상 200을 응답한다.
async fn health() -> &'static str {
    "ok"
}
