# 03: 원본 URL 검증

**What to build:** 서버가 원본 URL을 검증해, 유효한 `http`/`https` URL만 링크로 만든다. 잘못된 입력은 `400`과 사람이 읽을 수 있는 오류 메시지로 거부하고, 화면은 그 메시지를 보여 준다. 규칙은 `AGENTS.md`의 "원본 URL 검증"과 스펙의 "원본 URL 검증" 모듈을 따른다.

**Blocked by:** 02

**Status:** resolved

- [x] 앞뒤 공백을 제거한 뒤 검증한다
- [x] `ftp://`, `javascript:`, host 없는 URL, 2048자 초과, 빈 문자열, `PUBLIC_BASE_URL`과 같은 host는 `400`으로 거부되고 링크가 만들어지지 않는다 (경계 1 테스트)
- [x] 저장되는 원본 URL은 정규화한 값이며 경로·쿼리·프래그먼트가 보존된다
- [x] 정확히 2048자 경계값은 순수 함수 단위 테스트로 확인한다
- [x] 화면이 서버의 오류 메시지를 표시한다

## Comments

- 화면의 오류 메시지 표시는 `src/main.rs`에서 400 응답의 `error` 필드를 보여 주도록 구현했고, wasm clippy와 `dx build --release` 통과로 확인했다. 브라우저에서 직접 눈으로 확인하지는 않았다.
