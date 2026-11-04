//! 서버 API 라우터. Dioxus 서버에 합쳐 쓰고, 테스트에서는 단독으로 호출한다.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use sqlx::PgPool;

use crate::dto::{CreateLinkRequest, LinkResponse};
use crate::links;

/// 라우터가 공유하는 상태.
#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    public_base_url: String,
}

impl AppState {
    /// 커넥션 풀과 서비스 공개 주소(`PUBLIC_BASE_URL`)로 상태를 만든다.
    pub fn new(pool: PgPool, public_base_url: String) -> Self {
        Self {
            pool,
            // 끝의 `/`가 있어도 `<base>/<번호>`가 깔끔하게 만들어지도록 정리한다.
            public_base_url: public_base_url.trim_end_matches('/').to_string(),
        }
    }
}

/// 단축 서비스의 API 라우터를 만든다.
///
/// 주의: Dioxus 라우터가 이미 fallback을 가지므로 여기서 `.fallback()`을 달지 않는다.
pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/links", post(create_link))
        .route("/{segment}", get(redirect))
        .with_state(state)
}

/// 배포 환경의 헬스체크용. 항상 200을 응답한다.
async fn health() -> &'static str {
    "ok"
}

/// 링크를 발급한다. (검증은 티켓 03에서 추가한다.)
async fn create_link(
    State(state): State<AppState>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<(StatusCode, Json<LinkResponse>), StatusCode> {
    let link = links::insert(&state.pool, &req.url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = LinkResponse {
        number: link.number,
        short_url: format!("{}/{}", state.public_base_url, link.number),
        original_url: link.original_url,
    };
    Ok((StatusCode::CREATED, Json(body)))
}

/// 단축 번호로 원본 URL에 301 이동한다. (404와 번호 정규형 판정은 티켓 04에서 다룬다.)
async fn redirect(State(state): State<AppState>, Path(segment): Path<String>) -> Response {
    let Ok(number) = segment.parse::<i64>() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match links::find(&state.pool, number).await {
        Ok(Some(link)) => match HeaderValue::from_str(&link.original_url) {
            Ok(location) => (
                StatusCode::MOVED_PERMANENTLY,
                [(header::LOCATION, location)],
            )
                .into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
