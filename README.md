# 숫자형 단축 URL 서비스

## 배포 URL

- 서비스 URL: https://short-url.tbafe.workers.dev
- 테스트 단축 URL: https://short-url.tbafe.workers.dev/2
- 테스트 원본 URL: https://www.rust-lang.org/learn/get-started?lang=ko#installing-rust

로그인 없이 긴 URL을 `/123`처럼 숫자로만 된 단축 주소로 바꾸는 Dioxus 풀스택(Rust) 서비스입니다.

<p align="center">
  <img src="docs/images/result.png" alt="단축 주소를 만들고 복사한 뒤의 화면" width="520">
</p>

## 구성

<p align="center">
  <img src="docs/images/architecture.svg" alt="사용자 → Cloudflare Worker → Railway 백엔드 → PostgreSQL" width="100%">
</p>

| 구성 | 역할 | 설정 파일 |
| --- | --- | --- |
| Cloudflare Worker | 공개 주소. `/api/*`와 `/{숫자}`는 백엔드로 프록시하고, 나머지는 웹 화면을 제공 | `cloudflare/wrangler.jsonc`, `cloudflare/src/worker.js` |
| Railway 백엔드 | 생성 API, 301 이동, 404 안내, 변경 거부 | `Dockerfile`, `railway.json` |
| Railway PostgreSQL | 링크 저장 | `migrations/` |

**연결 방법**

- 브라우저 → Worker: 모든 요청은 Worker 주소 하나로 갑니다.
- Worker → 백엔드: `wrangler.jsonc`의 `BACKEND_ORIGIN`(Railway 공개 주소)으로 프록시합니다. 백엔드의 301 응답은 `redirect: "manual"`로 받아 그대로 전달합니다.
- 백엔드 → DB: Railway 변수 `DATABASE_URL`(`${{Postgres.DATABASE_URL}}` 참조)로 연결합니다.

## 설계

### 단축 번호 발급과 중복 방지

```sql
number BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY
```

- 번호는 DB만 발급합니다. 시퀀스가 원자적으로 나눠 주므로 동시에 요청이 들어와도 충돌하지 않습니다.
- `PRIMARY KEY`가 같은 번호의 중복 저장을 막고, `GENERATED ALWAYS`가 번호를 직접 지정한 `INSERT`를 거부합니다.
- 같은 원본 URL도 매번 새 번호를 받습니다.

### 생성 후 불변

링크는 생성만 할 수 있고, 변경·삭제·만료·재사용은 없습니다. 세 겹으로 막습니다.

1. **API**: `/api/links/{n}`의 `PUT`·`PATCH`·`DELETE`는 항상 `405`로 거부합니다.
2. **저장소 코드**: `src/links.rs`에는 `insert`와 `find`만 있습니다.
3. **DB**: 트리거가 `UPDATE`·`DELETE`·`TRUNCATE`를 거부합니다 (`migrations/0002_links_immutable.sql`).

### 이동과 입력 검증

- `GET /{n}`은 `301`로 응답하고, `Location`에 원본 URL을 그대로 넣어 경로·쿼리·프래그먼트를 보존합니다.
- 없는 번호와 `/0`, `/0123` 같은 비정규형 숫자는 `404` 안내 페이지로 응답합니다.
- 서버가 원본 URL을 검증합니다. `http`/`https`만, host 필수, 2048자 이하, 서비스 자신의 주소는 거부합니다. 실패하면 `400`과 안내 메시지를 줍니다.

## 환경변수

[`.env.example`](.env.example)에 이름만 있습니다.

| 변수 | 쓰는 곳 | 설명 |
| --- | --- | --- |
| `DATABASE_URL` | 백엔드 | Postgres 접속 주소 |
| `PUBLIC_BASE_URL` | 백엔드 | 서비스 공개 주소(Worker 주소). 단축 주소의 기준 |
| `IP`, `PORT` | 백엔드 | 바인드 주소. Railway가 `PORT`를 주입 |
| `BACKEND_ORIGIN` | Worker | Railway 백엔드 공개 주소 |

**비밀 값은 저장소에 없습니다.** DB 접속 정보는 Railway 변수로만 전달되고, `.env`는 `.gitignore`로 제외됩니다. 브라우저 번들과 `wrangler.jsonc`에는 공개 주소만 들어 있습니다.

## 실행

로컬 Postgres를 띄웁니다. 테스트 전용 컨테이너입니다.

```bash
docker run -d --name short-url-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17
```

서버와 화면을 띄웁니다. 스키마는 서버가 시작할 때 자동 적용됩니다. 준비물은 Rust(`wasm32-unknown-unknown` 타깃)와 Dioxus CLI 0.7.10입니다.

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8080 dx serve --fullstack
```

## 배포

**1. Railway (백엔드 + DB)**

1. **New Project → Deploy from GitHub repo**로 이 저장소를 연결합니다. `railway.json`에 따라 `Dockerfile`로 빌드합니다.
2. **+ Create → Database → PostgreSQL**을 추가합니다.
3. 백엔드 **Variables**에 `DATABASE_URL=${{Postgres.DATABASE_URL}}`, `PUBLIC_BASE_URL=<Worker 주소>`를 넣습니다.
4. **Settings → Networking → Generate Domain**으로 공개 주소를 만듭니다.

이후 `main`에 push하면 자동으로 다시 배포됩니다.

**2. Cloudflare (공개 주소 + 화면)**

웹 화면을 빌드합니다.

```bash
dx build --release --fullstack
```

Worker를 배포합니다. `wrangler.jsonc`의 `BACKEND_ORIGIN`에 Railway 주소를 넣어 둡니다.

```bash
npx wrangler deploy --config cloudflare/wrangler.jsonc
```

## 검증 결과

`cargo fmt --check`, `cargo clippy`(server·web), `cargo test --features server`(19개)가 모두 통과합니다.

배포 환경에서 확인한 결과입니다.

| 시나리오 | 자동 테스트 | 결과 |
| --- | --- | --- |
| 시크릿 창에서 생성하고 복사 | – | ✅ |
| 다른 브라우저에서 단축 주소로 이동 | `tests/issue_and_redirect.rs` | ✅ |
| 경로·쿼리·프래그먼트 보존 | `tests/issue_and_redirect.rs` | ✅ `/2` → `301`, 원본 그대로 |
| 서로 다른 URL 동시 생성 | `tests/concurrency_persistence.rs` | ✅ 20건 모두 다른 번호 |
| 재배포 후 이전 주소로 이동 | `tests/concurrency_persistence.rs` | ✅ |
| 잘못된 URL | `tests/url_validation.rs` | ✅ `400` + 안내 |
| 존재하지 않는 숫자 | `tests/not_found.rs` | ✅ `404` 안내 페이지 |
| 기존 연결 변경 요청 | `tests/immutability_api.rs`, `tests/immutability_db.rs` | ✅ `405`, 이동 결과 그대로 |
