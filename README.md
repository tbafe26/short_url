# 숫자형 단축 URL 서비스

## 배포 URL

- 서비스 URL: https://short-url.tbafe.workers.dev
- 테스트 단축 URL: https://short-url.tbafe.workers.dev/2
- 테스트 원본 URL: https://www.rust-lang.org/learn/get-started?lang=ko#installing-rust

> 로그인 없이 긴 URL을 `/123`처럼 **숫자로만 된 단축 주소**로 바꾸는 Dioxus 풀스택(Rust) 서비스입니다.
> 한 번 발급한 단축 번호와 원본 URL의 연결(링크)은 **바꾸거나 지울 수 없습니다.**

<p align="center">
  <img src="docs/images/result.png" alt="단축 주소를 만들고 복사한 뒤의 화면" width="520">
  <br>
  <sub>원본 URL을 넣고 <b>단축하기</b> → 단축 주소와 원본 표시 → <b>복사</b></sub>
</p>

## 목차

1. [주요 기능](#주요-기능)
2. [아키텍처](#아키텍처)
3. [설계](#설계)
4. [API](#api)
5. [프로젝트 구조](#프로젝트-구조)
6. [로컬 실행](#로컬-실행)
7. [배포](#배포)
8. [검증](#검증)
9. [관련 문서](#관련-문서)

## 주요 기능

| 요구사항 | 구현 | 코드 |
| --- | --- | --- |
| 로그인 없이 생성·이동 | 인증 없는 공개 API와 화면 | `src/api.rs`, `src/main.rs` |
| 입력·생성·결과 표시·복사 화면 | Dioxus 웹 화면(WASM). Enter로도 생성, 클립보드 복사 | `src/main.rs`, `assets/main.css` |
| 숫자로만 된 단축 경로 | DB의 `IDENTITY` 기본키로만 번호 발급 | `migrations/0001_create_links.sql` |
| 번호 충돌·중복 방지 | DB 시퀀스 + `PRIMARY KEY` + `GENERATED ALWAYS` | 같은 파일 |
| 변경·삭제·만료·재사용 없음 | API 405 + 저장소 코드 + DB 트리거, 세 겹 | `src/api.rs`, `src/links.rs`, `migrations/0002_links_immutable.sql` |
| `/숫자` → HTTP 리디렉션 | `301` + 원본 URL 그대로(경로·쿼리·프래그먼트 보존) | `src/api.rs` `redirect` |
| 서버 측 입력 검증 | `http`/`https`만, host 필수, 2048자 이하 | `src/validation.rs` |
| 없는 번호는 404 | 읽을 수 있는 안내 페이지 | `src/api.rs` `not_found` |
| 데이터 영구 저장 | Railway PostgreSQL 17 | `migrations/` |

## 아키텍처

<p align="center">
  <img src="docs/images/architecture.svg" alt="아키텍처: 사용자 → Cloudflare Worker → Railway 백엔드 → PostgreSQL" width="100%">
</p>

| 구성 요소 | 플랫폼 | 역할 | 설정 파일 |
| --- | --- | --- | --- |
| Worker 스크립트 | Cloudflare Workers | 공개 HTTPS 접점. `/api/*`와 `/{숫자}`는 백엔드로 프록시, 나머지는 정적 파일 | `cloudflare/wrangler.jsonc`, `cloudflare/src/worker.js` |
| 웹 화면 | Cloudflare Static Assets | Dioxus 웹 번들(index.html, WASM, CSS) | `src/main.rs`, `assets/main.css`, `index.html`, `Dioxus.toml` |
| 백엔드 | Railway (Docker) | 생성 API, 301 이동, 404 안내, 변경 거부 | `Dockerfile`, `railway.json`, `src/` |
| DB | Railway PostgreSQL 17 | 링크 영구 저장, 번호 발급, 불변성 강제 | `migrations/` |

**요청 흐름**

- **생성**: 브라우저 → Worker `POST /api/links` → 백엔드가 URL 검증 → DB `INSERT`가 번호 발급 → `201` + 단축 주소
- **이동**: 브라우저 → Worker `GET /27` → 백엔드가 DB 조회 → `301 Location: <원본 URL>` → 브라우저가 원본으로 이동

**설계 포인트**

- **같은 출처**: 브라우저는 Worker 주소로만 요청합니다. 그래서 CORS가 필요 없고, 백엔드 주소가 화면 코드에 들어가지 않습니다 (ADR-0001).
- **화면과 서버를 한 크레이트에**: `web` 기능으로 화면(WASM)을, `server` 기능으로 서버를 빌드합니다. 화면은 Worker가 정적 파일로 주므로 웹 빌드에는 hydration(`dioxus/fullstack`)을 넣지 않습니다.

<details>
<summary><b>Worker 라우팅 규칙</b> (<code>cloudflare/src/worker.js</code>)</summary>

- `/api`, `/api/*`, 숫자로만 된 단일 세그먼트(`/^\/[0-9]+$/`) → Railway 백엔드로 프록시
  - `fetch(..., { redirect: "manual" })`로 받아 백엔드의 `301`과 `Location`을 그대로 브라우저에 전달합니다.
  - 정규형 판정(`/0`, `/0123` → 404)은 백엔드가 합니다.
  - Railway 절전에서 깨어나는 첫 GET이 `502`/`503`이면 한 번 다시 시도합니다.
- 그 밖의 경로 → `env.ASSETS.fetch`로 웹 화면 정적 파일
- `not_found_handling`을 SPA로 두지 않습니다. 그러면 브라우저에서 연 `/123`이 Worker 스크립트를 거치지 않기 때문입니다.

</details>

### 기술 스택

| 영역 | 사용 기술 |
| --- | --- |
| 언어·프레임워크 | Rust, Dioxus 0.7.10 (풀스택: 웹 WASM + 서버) |
| 서버 | axum 0.8, tokio |
| DB | PostgreSQL 17, sqlx 0.9 (마이그레이션 내장) |
| 공개 접점 | Cloudflare Workers + Static Assets (`workers.dev`) |
| 백엔드 호스팅 | Railway (Dockerfile 빌드, Trial 크레딧) |

## 설계

### 단축 번호 발급과 중복 방지

```sql
CREATE TABLE links (
    number       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    original_url TEXT NOT NULL CHECK (char_length(original_url) <= 2048),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

- **번호는 DB만 발급합니다.** 애플리케이션은 번호를 계산하거나 지정하지 않습니다. 시퀀스가 원자적으로 나눠 주므로 동시에 요청이 들어와도 충돌하지 않습니다.
- **`PRIMARY KEY`**(UNIQUE 포함)가 같은 번호의 두 번째 저장을 한 번 더 막습니다.
- **`GENERATED ALWAYS`**라서 번호를 직접 지정한 `INSERT`도 거부됩니다. 이미 쓰인 번호를 다른 URL에 다시 쓸 수 없습니다.
- **같은 원본 URL은 매번 새 번호를 받습니다.** 기존 번호를 돌려주려면 긴 URL에 UNIQUE(해시 컬럼)를 걸고 동시 입력 경쟁까지 처리해야 하므로, 단순한 규칙을 택했습니다 (ADR-0002).
- **정규형만 인정합니다.** 1 이상, 앞자리 0 없음, `i64` 범위 안. `/0`, `/0123`, 범위를 넘는 숫자는 404입니다. 그래서 링크 하나에 단축 주소는 정확히 하나입니다.

### 생성 후 불변: 세 겹 방어 (ADR-0002)

| 층 | 방법 |
| --- | --- |
| 1. API | 생성 경로만 있습니다. `/api/links/{n}`의 `PUT`·`PATCH`·`DELETE`는 링크 존재 여부와 상관없이 `405`로 거부합니다. 화면에 버튼이 없는 것에 그치지 않고 서버가 거부합니다. |
| 2. 저장소 코드 | `src/links.rs`의 공개 함수는 `insert`와 `find`뿐입니다. |
| 3. DB | `BEFORE UPDATE OR DELETE` 행 트리거와 `BEFORE TRUNCATE` 문장 트리거가 SQL로 직접 시도한 변경도 막습니다. |

수정, 삭제, 자동 만료 기능은 없습니다.

### 리디렉션

`GET /{n}`은 `301 Moved Permanently`로 응답하고, `Location`에 저장된 원본 URL을 **그대로** 넣습니다. 경로·쿼리·프래그먼트가 모두 보존됩니다.

```
GET /2  →  301  Location: https://www.rust-lang.org/learn/get-started?lang=ko#installing-rust
```

### 원본 URL 검증 (서버)

`url` 크레이트로 파싱한 뒤 아래 조건을 **모두** 통과해야 링크로 만듭니다. 저장은 정규화한 값으로 합니다. 화면의 검증은 편의 기능이고, 판단은 서버가 합니다.

- 앞뒤 공백 제거
- scheme이 `http` 또는 `https`
- host가 있음
- 2048자 이하
- 서비스 자신의 host(`PUBLIC_BASE_URL`)가 아님 (단축 주소를 다시 단축하는 것을 막음)

실패하면 `400`과 `{"error": "<안내 메시지>"}`로 응답하고, 화면은 그 메시지를 그대로 보여 줍니다.

## API

| 요청 | 응답 |
| --- | --- |
| `POST /api/links` `{"url": "<원본 URL>"}` | `201` `{"number", "short_url", "original_url"}` · 검증 실패나 본문 형식 오류는 `400` `{"error"}` |
| `GET /{n}` | 링크가 있으면 `301` + `Location: <원본 URL>` · 없거나 정규형이 아니면 `404` 안내 HTML |
| `PUT`·`PATCH`·`DELETE /api/links/{n}` | 항상 `405` + 불변성 안내 `{"error"}` |
| `GET /api/health` | `200 ok` (Railway 헬스 체크) |

## 프로젝트 구조

```
short_url/
├── src/
│   ├── main.rs           # 진입점 + 생성 화면(Dioxus 컴포넌트)
│   ├── api.rs            # API 라우터: 생성, 301 이동, 404, 405
│   ├── links.rs          # 저장소: insert, find (변경 함수 없음)
│   ├── validation.rs     # 원본 URL 검증
│   ├── short_number.rs   # 단축 번호 정규형 판정
│   └── dto.rs            # 요청·응답 타입
├── migrations/           # 테이블 생성, 불변성 트리거
├── tests/                # API·DB 통합 테스트 (19개)
├── assets/main.css       # 화면 스타일
├── index.html            # 웹 번들 HTML 템플릿
├── Dioxus.toml           # dx 설정 (페이지 제목)
├── cloudflare/           # Worker 스크립트와 wrangler 설정
├── Dockerfile            # Railway 백엔드 이미지 (멀티스테이지)
├── railway.json          # Railway 빌드·헬스 체크 설정
├── .env.example          # 환경변수 이름 (값 없음)
└── docs/                 # ADR, 정리 목록, 이미지
```

## 로컬 실행

### 준비물

- Rust stable + `wasm32-unknown-unknown` 타깃 (Windows는 MSVC Build Tools(C++) 필요)
- Dioxus CLI 0.7.10: `cargo install dioxus-cli --version 0.7.10 --locked`
- Docker (로컬 Postgres)
- Node.js (Cloudflare `wrangler`)

### 1. DB 준비

```bash
docker run -d --name short-url-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17
```

로컬 테스트 전용 컨테이너라 비밀번호는 기본값을 씁니다. 스키마는 서버가 시작할 때 `sqlx::migrate!`로 자동 적용됩니다.

### 2. 환경변수

[`.env.example`](.env.example)에 **이름만** 있습니다. 실제 값은 셸 환경변수나 배포 환경에 넣습니다.

| 변수 | 쓰는 곳 | 설명 |
| --- | --- | --- |
| `DATABASE_URL` | 백엔드, 테스트 | Postgres 접속 주소. Railway에서는 `${{Postgres.DATABASE_URL}}` 참조 |
| `PUBLIC_BASE_URL` | 백엔드 | 서비스 공개 주소(Worker 주소, 끝에 `/` 없이). 단축 주소의 기준 |
| `IP`, `PORT` | 백엔드 | 바인드 주소. 이미지는 `IP=0.0.0.0`이고 Railway가 `PORT`를 주입 |
| `BACKEND_ORIGIN` | Worker | Railway 백엔드 공개 주소. `wrangler.jsonc`의 `vars`나 `--var`로 지정 |

### 3. 실행

**개발 서버 (백엔드 + 화면)**

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8080 dx serve --fullstack
```

**배포와 같은 구성 (release 서버 + 로컬 Worker)**

release로 빌드합니다.

```bash
dx build --release --fullstack
```

빌드 결과 폴더에서 서버를 띄웁니다. Windows에서는 `server.exe`입니다.

```bash
cd target/dx/short_url/release/web && IP=127.0.0.1 PORT=8081 DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8787 ./server
```

다른 터미널에서, 저장소 루트를 기준으로 Worker를 띄우고 `http://localhost:8787`을 엽니다.

```bash
npx wrangler dev --config cloudflare/wrangler.jsonc --port 8787 --var BACKEND_ORIGIN:http://127.0.0.1:8081
```

## 배포

### 1. Railway (백엔드 + DB)

1. Railway에 GitHub 계정으로 가입합니다. 카드는 등록하지 않습니다.
2. **New Project → Deploy from GitHub repo**에서 이 저장소를 고릅니다. 루트의 `railway.json` 때문에 `Dockerfile` 빌드와 `/api/health` 헬스 체크가 적용됩니다.
3. **+ Create → Database → PostgreSQL**을 추가합니다.
4. 백엔드 서비스 **Variables**에 아래 두 변수를 넣습니다.
   - `DATABASE_URL` = `${{Postgres.DATABASE_URL}}`
   - `PUBLIC_BASE_URL` = Worker 주소 (Cloudflare 배포 후)
5. **Settings → Networking → Generate Domain**으로 `https://<백엔드>.up.railway.app`을 만듭니다.
6. (선택) **Settings → Deploy → Serverless**를 켜면 유휴 시간에 절전해 크레딧을 아낍니다. Postgres에는 켜지 않습니다.

이후에는 `main`에 push하면 Railway가 자동으로 다시 빌드·배포합니다. `Dockerfile`은 멀티스테이지로, 1단계에서 `dx build --release --fullstack`으로 빌드하고 2단계에서 실행 파일과 `public/`만 `debian:trixie-slim`에 담습니다.

### 2. Cloudflare (공개 접점)

출력 폴더를 지우고 깨끗하게 다시 빌드합니다.

```bash
rm -rf target/dx/short_url/release/web && dx build --release --fullstack
```

Cloudflare에 로그인합니다.

```bash
npx wrangler login
```

배포합니다. `wrangler.jsonc`의 `vars.BACKEND_ORIGIN`에 지금 쓰는 백엔드 주소가 들어 있습니다.

```bash
npx wrangler deploy --config cloudflare/wrangler.jsonc
```

배포가 끝나면 `https://short-url.<하위 도메인>.workers.dev`가 출력됩니다. 이 주소를 Railway의 `PUBLIC_BASE_URL`에 넣습니다. 다른 백엔드를 가리키려면 `--var BACKEND_ORIGIN:https://<백엔드>.up.railway.app`을 붙입니다.

### 비밀 값 관리

- **저장소에는 비밀 값이 없습니다.** 커밋 이력 전체를 검사해 확인했습니다.
- DB 접속 정보는 Railway 변수에서 `${{Postgres.DATABASE_URL}}` **참조**로만 전달됩니다. 실제 값은 Railway 밖으로 나오지 않습니다.
- `.env`는 `.gitignore`로 제외되고, 저장소에는 이름만 있는 `.env.example`만 있습니다.
- 브라우저 번들(WASM)과 `cloudflare/wrangler.jsonc`에는 공개 주소(`BACKEND_ORIGIN`)만 들어 있습니다.

### 비용

운영 비용은 **0원**입니다 (ADR-0003). Railway는 카드 없이 Trial 크레딧 안에서만 운영합니다. 크레딧이 떨어지면 서비스가 멈출 뿐 청구되지 않습니다. Cloudflare는 무료 플랜과 무료 `workers.dev` 주소를 씁니다. 정리 방법은 [`docs/cleanup.md`](docs/cleanup.md)에 있습니다.

## 검증

### 검증 명령

```bash
cargo fmt --all -- --check
cargo clippy --features server --all-targets -- -D warnings
cargo clippy --features web --target wasm32-unknown-unknown -- -D warnings
cargo test --features server
dx build --release --fullstack
npx wrangler deploy --dry-run --config cloudflare/wrangler.jsonc
```

모두 통과합니다. `cargo test`(19개)에는 `DATABASE_URL`이 필요하며, `#[sqlx::test]`가 테스트마다 격리된 DB를 만들고 마이그레이션을 적용합니다.

### 확인 시나리오와 결과

배포 환경(`https://short-url.tbafe.workers.dev`, 2026-11-05)에서 확인한 결과입니다.

| 시나리오 | 자동 테스트 | 배포 환경 결과 |
| --- | --- | --- |
| 시크릿 창에서 생성하고 복사 | – (화면) | ✅ 생성 → 복사 → "복사했습니다" (직접 확인) |
| 다른 브라우저에서 단축 주소로 이동 | `tests/issue_and_redirect.rs` | ✅ 복사한 주소로 원본 이동 (직접 확인) |
| 경로·쿼리·프래그먼트 보존 | `tests/issue_and_redirect.rs` | ✅ `/2` → `301 Location: …/get-started?lang=ko#installing-rust` |
| 서로 다른 URL 동시 생성 시 충돌 없음 | `tests/concurrency_persistence.rs` (50건) | ✅ 20건 동시 생성 → 모두 `201`, 번호 20개 모두 다름, 각자 원본으로 이동 |
| 재시작·재배포 후 이전 주소 유지 | `tests/concurrency_persistence.rs` | ✅ 재배포 후 `/1`, `/2`, `/3`, `/23` 그대로 `301`, 새 번호는 이어서 발급 |
| 잘못된 URL | `tests/url_validation.rs` | ✅ `javascript:alert(1)`, `ftp://x.org/a`, `not a url`, 서비스 자신의 주소 → `400` + 안내 |
| 존재하지 않는 숫자 | `tests/not_found.rs` | ✅ `/999999999`, `/0123` → `404` 안내 페이지 |
| 기존 연결 변경 요청 | `tests/immutability_api.rs`, `tests/immutability_db.rs` | ✅ `PUT`·`PATCH`·`DELETE` → `405`, 이후에도 `/2` 이동 결과 그대로 |

<details>
<summary><b>직접 재현하기</b> (curl)</summary>

생성:

```bash
curl -i -X POST https://short-url.tbafe.workers.dev/api/links -H 'content-type: application/json' -d '{"url":"https://example.com/a?b=1#c"}'
```

이동 (경로·쿼리·프래그먼트 확인):

```bash
curl -i https://short-url.tbafe.workers.dev/2
```

동시 생성 20건:

```bash
for i in $(seq 1 20); do curl -s -X POST https://short-url.tbafe.workers.dev/api/links -H 'content-type: application/json' -d "{\"url\":\"https://example.com/$i\"}" & done; wait
```

변경 요청 (→ `405`):

```bash
curl -i -X PUT https://short-url.tbafe.workers.dev/api/links/1 -H 'content-type: application/json' -d '{"url":"https://evil.example"}'
```

없는 번호 (→ `404`):

```bash
curl -i https://short-url.tbafe.workers.dev/999999999
```

</details>

## 관련 문서

| 문서 | 내용 |
| --- | --- |
| [`GLOSSARY.md`](GLOSSARY.md) | 도메인 용어 (원본 URL, 단축 번호, 단축 주소, 링크, 발급, 이동) |
| [`docs/adr/0001`](docs/adr/0001-cloudflare-worker-as-public-entry.md) | Cloudflare Worker를 유일한 공개 접점으로 |
| [`docs/adr/0002`](docs/adr/0002-link-immutability-in-three-layers.md) | 링크 불변성을 세 겹으로 |
| [`docs/adr/0003`](docs/adr/0003-zero-cost-operation.md) | 비용 0원 운영 |
| [`docs/cleanup.md`](docs/cleanup.md) | 설치한 도구와 클라우드 리소스 정리 방법 |
| [`AGENTS.md`](AGENTS.md) | 개발 규칙과 검증 명령 |
