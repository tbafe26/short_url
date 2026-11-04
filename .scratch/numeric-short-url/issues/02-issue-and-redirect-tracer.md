# 02: 링크 발급과 이동의 최소 경로

**What to build:** 방문자가 화면에서 원본 URL을 입력하고 생성 버튼을 누르면 단축 주소가 표시된다. 그 단축 주소(`/{n}`)에 접속하면 301로 원본 URL에 이동한다. 링크는 Postgres에 저장되고, 단축 번호는 DB identity로 발급된다. 검증·404·불변성은 이후 티켓에서 다루며, 이 티켓은 행복 경로만 끝까지 뚫는다. 스펙: `.scratch/numeric-short-url/spec.md` (구현 결정의 모듈·API 계약·스키마 참고).

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] `links` 테이블이 마이그레이션으로 만들어지고, 서버 시작 시 자동 적용된다 (`number BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY`, `original_url`, `created_at`)
- [ ] 링크 저장소의 공개 인터페이스는 `insert`와 `find`뿐이다
- [ ] 생성 API가 스펙의 계약대로 `201`과 `number`·`short_url`·`original_url`을 응답한다 (`short_url`은 `PUBLIC_BASE_URL` 기준)
- [ ] `GET /{n}`이 존재하는 링크에 대해 `301`과 `Location: <원본 URL>`을 응답한다
- [ ] 화면에서 생성 후 단축 주소가 표시된다
- [ ] 경계 1 테스트 하네스(실제 `Router`를 `oneshot`으로 호출, 테스트별 격리 DB)가 있고, 생성→이동 행복 경로 테스트가 통과한다
- [ ] `.env.example`에 백엔드 환경변수 이름이 있다 (값 없음)
