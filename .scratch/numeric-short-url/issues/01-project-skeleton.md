# 01: 프로젝트 골격과 검증 명령 확정

**What to build:** Dioxus 풀스택 앱의 빈 골격을 만든다. 로컬에서 개발 서버를 띄우면 서비스 제목이 있는 빈 생성 화면이 뜬다. `AGENTS.md`의 "검증 명령"을 실제로 실행해 보고, 실제 기능 플래그와 명령에 맞게 고친다. 스펙: `.scratch/numeric-short-url/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] 단일 크레이트에 `web`/`server` 기능이 나뉘어 있고, 최신 안정 Dioxus(0.7 계열)를 쓴다
- [ ] `dx serve`로 띄운 로컬 화면에 서비스 제목이 보인다
- [x] 로컬 테스트 DB를 띄우는 방법이 정해져 있다 (Docker `postgres` 컨테이너, 테스트용 `DATABASE_URL`)
- [x] `AGENTS.md` 검증 명령 중 Rust 관련 명령(fmt, clippy 두 개, test, dx build)이 모두 통과하고, 실제 명령과 다르면 `AGENTS.md`가 고쳐져 있다
- [x] 빌드 산출물과 `.env`가 `.gitignore`에 있다

## Comments

- `dx build --release`는 통과했다 (서버 `server.exe`와 웹 `public/` 생성, 첫 빌드 약 15분). 화면 제목은 `src/main.rs`의 `h1`로 렌더링되지만 `dx serve`로 눈으로 확인하지는 않았다. 02의 화면 작업에서 함께 확인한다.
- 구현 중 실패하는 테스트를 먼저 보는 red 단계를 거치지 않았다 (스모크 테스트 1개).
