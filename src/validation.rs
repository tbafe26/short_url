//! 원본 URL 검증. 입력 문자열을 정규화된 원본 URL로 바꾸는 순수 함수다.

use std::fmt;

use url::Url;

/// 원본 URL의 최대 길이(정규화한 값 기준).
pub const MAX_URL_LEN: usize = 2048;

/// 원본 URL 검증 오류. `Display`가 사람이 읽을 수 있는 메시지다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlError {
    /// 빈 입력
    Empty,
    /// URL로 해석할 수 없음
    Invalid,
    /// `http`/`https`가 아닌 scheme
    UnsupportedScheme,
    /// host가 없음
    MissingHost,
    /// 정규화한 값이 2048자 초과
    TooLong,
    /// 서비스 자신의 주소를 가리킴
    SelfReference,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "원본 URL을 입력해 주세요.",
            Self::Invalid => "올바른 URL 형식이 아닙니다.",
            Self::UnsupportedScheme => "http 또는 https 주소만 사용할 수 있습니다.",
            Self::MissingHost => "주소에 host가 있어야 합니다.",
            Self::TooLong => "원본 URL은 2048자 이하여야 합니다.",
            Self::SelfReference => "이 서비스의 주소는 단축할 수 없습니다.",
        };
        f.write_str(message)
    }
}

/// 원본 URL을 검증하고 정규화한(직렬화한) 값을 돌려준다.
///
/// `public_base_url`은 서비스 공개 주소(`PUBLIC_BASE_URL`)로, 같은 host는 거부한다.
pub fn validate_original_url(input: &str, public_base_url: &str) -> Result<String, UrlError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(UrlError::Empty);
    }
    let url = Url::parse(trimmed).map_err(|_| UrlError::Invalid)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(UrlError::UnsupportedScheme);
    }
    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or(UrlError::MissingHost)?;
    if url.as_str().chars().count() > MAX_URL_LEN {
        return Err(UrlError::TooLong);
    }
    if let Ok(base) = Url::parse(public_base_url)
        && base.host_str() == Some(host)
    {
        return Err(UrlError::SelfReference);
    }
    Ok(url.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://short.test";

    /// 정규화한 결과가 정확히 `len`자가 되는 URL을 만든다.
    fn url_of_len(len: usize) -> String {
        let prefix = "https://example.com/";
        format!("{prefix}{}", "a".repeat(len - prefix.len()))
    }

    #[test]
    fn 정확히_2048자는_통과한다() {
        let input = url_of_len(MAX_URL_LEN);
        assert_eq!(validate_original_url(&input, BASE), Ok(input));
    }

    #[test]
    fn 정확히_2049자는_거부한다() {
        let input = url_of_len(MAX_URL_LEN + 1);
        assert_eq!(validate_original_url(&input, BASE), Err(UrlError::TooLong));
    }
}
