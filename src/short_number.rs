//! 단축 번호 정규형 파싱.

/// 경로 조각이 정규형 단축 번호이면 번호를 돌려준다.
///
/// 정규형: 1 이상, 앞자리 0 없음, `i64` 범위 안. 그 밖에는 `None`.
pub fn parse_short_number(segment: &str) -> Option<i64> {
    // 숫자 문자(ASCII)만, 첫 자리는 1~9여야 한다. 부호·공백·앞자리 0은 정규형이 아니다.
    let mut bytes = segment.bytes();
    let first = bytes.next()?;
    if !(b'1'..=b'9').contains(&first) || !bytes.all(|b| b.is_ascii_digit()) {
        return None;
    }
    // i64 범위를 넘으면 파싱이 실패해 None이 된다.
    segment.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_최댓값은_인정하고_다음_값은_거부한다() {
        assert_eq!(parse_short_number("9223372036854775807"), Some(i64::MAX));
        assert_eq!(parse_short_number("9223372036854775808"), None);
    }

    #[test]
    fn 정규형이_아닌_값은_거부한다() {
        for s in ["0", "0123", "+1", "-1", "", "1a", " 1", "１"] {
            assert_eq!(parse_short_number(s), None, "{s:?}");
        }
        assert_eq!(parse_short_number("1"), Some(1));
    }
}
