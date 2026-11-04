//! 서버와 브라우저가 함께 쓰는 API 요청·응답 형식.

use serde::{Deserialize, Serialize};

/// 링크 생성 요청.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLinkRequest {
    /// 사용자가 입력한 원본 URL
    pub url: String,
}

/// 링크 생성 성공 응답.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
    /// 단축 번호
    pub number: i64,
    /// 단축 주소 (`PUBLIC_BASE_URL/<번호>`)
    pub short_url: String,
    /// 저장된 원본 URL
    pub original_url: String,
}
