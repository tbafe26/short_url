# 01: 프로젝트 골격과 검증 명령 확정

**What to build:** Dioxus 풀스택 앱의 빈 골격을 만든다. 로컬에서 개발 서버를 띄우면 서비스 제목이 있는 빈 생성 화면이 뜬다. `AGENTS.md`의 "검증 명령"을 실제로 실행해 보고, 실제 기능 플래그와 명령에 맞게 고친다. 스펙: `.scratch/numeric-short-url/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] 단일 크레이트에 `web`/`server` 기능이 나뉘어 있고, 최신 안정 Dioxus(0.7 계열)를 쓴다
- [ ] `dx serve`로 띄운 로컬 화면에 서비스 제목이 보인다
- [ ] 로컬 테스트 DB를 띄우는 방법이 정해져 있다 (Docker `postgres` 컨테이너, 테스트용 `DATABASE_URL`)
- [ ] `AGENTS.md` 검증 명령 중 Rust 관련 명령(fmt, clippy 두 개, test, dx build)이 모두 통과하고, 실제 명령과 다르면 `AGENTS.md`가 고쳐져 있다
- [ ] 빌드 산출물과 `.env`가 `.gitignore`에 있다
