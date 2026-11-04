# AGENTS.md

Dioxus 풀스택(Rust)으로 만드는 숫자형 단축 URL 서비스. 웹 화면은 Cloudflare Worker, 백엔드와 DB는 Railway에 배포한다.

도메인 용어는 `GLOSSARY.md`를 따른다. 코드·주석·커밋에서도 같은 용어(원본 URL, 단축 번호, 단축 주소, 링크, 발급, 이동)를 쓴다. 설계 이유는 `docs/adr/`에 있다.

## 개발 규칙

### 링크는 불변이다

링크는 생성만 한다. 불변성은 세 겹으로 지킨다 (ADR-0002).

- **API**: 생성 경로만 둔다. `/api/links/{n}`의 `PUT`/`PATCH`/`DELETE`는 405로 거부하는 핸들러를 유지한다.
- **저장소 코드**: `insert`와 `find`만 둔다.
- **DB**: `BEFORE UPDATE OR DELETE` 트리거로 변경을 거부한다.

새 기능이 기존 링크를 바꿔야 한다면 구현을 멈추고 사용자에게 묻는다.

### 단축 번호

- 단축 번호는 DB의 `BIGINT GENERATED ALWAYS AS IDENTITY` 기본키로만 발급한다. 애플리케이션 코드에서 번호를 계산하지 않는다.
- 같은 원본 URL이 다시 들어와도 매번 새 번호를 발급한다.
- 정규형만 인정한다: 1 이상, 앞자리 0 없음, `i64` 범위 안. 나머지 숫자 경로(`/0`, `/0123`, 범위 초과)와 없는 번호는 404 안내 페이지로 응답한다.

### 원본 URL 검증 (서버에서)

`url` 크레이트로 파싱한 뒤, 아래 조건을 모두 통과한 값만 받는다.

- scheme이 `http` 또는 `https`
- host가 있음
- 2048자 이하
- `PUBLIC_BASE_URL`과 같은 host가 아님

저장은 직렬화한(정규화된) 값으로 한다. 클라이언트 쪽 검증은 편의 기능일 뿐이고, 판단은 서버가 한다.

### 이동

301로 응답하고, `Location`에 저장된 원본 URL을 그대로 넣는다. 경로·쿼리·프래그먼트를 보존해야 한다.

### 비밀 값과 비용

- DB 접속 정보와 Railway 주소는 환경변수로만 받는다. 저장소에는 `.env.example`에 변수 이름만 둔다.
- 브라우저 번들(WASM)과 `wrangler.jsonc`에는 공개해도 되는 값만 넣는다.
- 유료 플랜, 유료 도메인, 카드 등록이 필요한 설정은 추가하지 않는다 (ADR-0003). 필요하면 사용자에게 묻는다.
- 새 개발 도구·Docker 컨테이너·클라우드 리소스를 설치하거나 만들면 `docs/cleanup.md`에 정리 방법과 함께 적는다.

## 검증 명령

작업을 끝났다고 보고하기 전에 아래 명령이 모두 통과해야 한다. 통과하지 못한 명령이 있으면 출력과 함께 보고한다.

```bash
cargo fmt --all -- --check
cargo clippy --features server --all-targets -- -D warnings
cargo clippy --features web --target wasm32-unknown-unknown -- -D warnings
cargo test --features server
dx build --release
npx wrangler deploy --dry-run --config cloudflare/wrangler.jsonc
```

- `cargo test`의 DB 통합 테스트는 로컬 Postgres를 가리키는 `DATABASE_URL`이 필요하다. `#[sqlx::test]`가 테스트마다 DB를 만들므로 DB 생성 권한이 있는 계정을 쓴다.
  - 로컬 Postgres: `docker run -d --name short-url-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17`
  - `DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url` (셸 환경변수나 `.env`로 지정)
- 기능 플래그: `server`(서버·DB), `web`(브라우저 WASM). `default`는 비어 있고 `dx`가 플랫폼에 맞춰 자동으로 켠다.
- `dx build --release`는 서버와 WASM을 모두 release로 빌드해 첫 실행에 15분 정도 걸린다. 셸 타임아웃을 넉넉히 잡고 백그라운드로 돌린다.
- `dx serve`/`cargo test` 전에 서로의 `target` 잠금을 피하려면 `dx serve`를 먼저 끈다.
- API는 `src/api.rs`의 `api_router()`에 모으고, `src/main.rs`가 Dioxus 서버 라우터에 합친다. 테스트는 이 라우터를 `tower::ServiceExt::oneshot`으로 직접 호출한다.
- DB 통합 테스트에는 다음 시나리오를 포함한다.
  - 동시 생성 요청 시 번호 충돌이 없음
  - UPDATE/DELETE가 트리거에 거부됨
  - 이동 시 경로·쿼리·프래그먼트가 보존됨
- 프로젝트 구조나 기능 플래그가 바뀌어 위 명령이 달라지면 이 섹션을 함께 고친다.

## Agent skills

### Issue tracker

이슈와 스펙은 `.scratch/` 아래 로컬 마크다운 파일로 관리한다. See `docs/agents/issue-tracker.md`.

### Triage labels

기본 트리아지 라벨 5종(`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`)을 그대로 사용한다. See `docs/agents/triage-labels.md`.

### Domain docs

single-context: 루트의 `GLOSSARY.md` 하나와 `docs/adr/`. See `docs/agents/domain.md`.
