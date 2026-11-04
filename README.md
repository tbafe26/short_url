# 숫자형 단축 URL 서비스

## 배포 URL

- 서비스 URL: https://short-url.tbafe.workers.dev
- 테스트 단축 URL: https://short-url.tbafe.workers.dev/2
- 테스트 원본 URL: https://www.rust-lang.org/learn/get-started?lang=ko#installing-rust

로그인 없이 긴 URL을 `/123`처럼 숫자로만 된 단축 주소로 바꾸는 서비스다. 한 번 발급한 단축 번호와 원본 URL의 연결(링크)은 바꾸거나 지울 수 없다. 용어는 [GLOSSARY.md](GLOSSARY.md)를, 설계 결정은 [docs/adr/](docs/adr/)를 따른다.

## 구성

```
브라우저 ──HTTPS──▶ Cloudflare Worker (workers.dev)
                     ├─ /, /assets/*  → Dioxus 웹 화면 (Static Assets)
                     └─ /api/*, /123  → Railway 백엔드로 프록시 (301 응답은 그대로 전달)
                                          │
                                          ▼
                              Railway: Rust 백엔드 (Dioxus 풀스택 서버 + axum) ──▶ Railway Postgres
```

| 위치 | 역할 | 주요 파일 |
| --- | --- | --- |
| Cloudflare Worker | 공개 HTTPS 접점, 웹 화면 제공, API·이동 프록시 | `cloudflare/wrangler.jsonc`, `cloudflare/src/worker.js` |
| Railway 백엔드 | 링크 생성 API, 301 이동, 404 안내, 변경 거부 | `Dockerfile`, `railway.json`, `src/` |
| Railway Postgres | 링크 영구 저장 | `migrations/` |

- **같은 출처**: 브라우저는 Worker 주소로만 요청한다. 그래서 CORS 설정이 필요 없고, 백엔드 주소가 화면 코드에 들어가지 않는다 (ADR-0001).
- **화면과 서버를 한 크레이트에**: `web` 기능으로 화면(WASM)을, `server` 기능으로 서버를 빌드한다. 화면은 Worker가 정적 파일로 주므로, 웹 빌드에는 hydration(`dioxus/fullstack`)을 넣지 않는다.

### API

| 요청 | 응답 |
| --- | --- |
| `POST /api/links` `{"url": "<원본 URL>"}` | `201` `{"number", "short_url", "original_url"}`. 검증 실패는 `400` `{"error": "<메시지>"}` |
| `GET /{n}` | 정규형 번호이고 링크가 있으면 `301` + `Location: <원본 URL>`. 그 밖에는 `404` 안내 HTML |
| `PUT`·`PATCH`·`DELETE /api/links/{n}` | 항상 `405` + `Allow` 헤더(빈 값) + 불변성 설명 |
| `GET /api/health` | `200 ok` (Railway 헬스 체크) |

## 설계: 번호 발급, 중복 방지, 불변성

### 숫자 발급 방식

- 단축 번호는 Postgres의 `number BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY`로만 발급한다. 애플리케이션은 번호를 계산하거나 지정하지 않는다.
- 1부터 순서대로 발급한다. 트랜잭션이 실패하면 번호에 구멍이 생길 수 있지만, 같은 번호가 두 번 나오지는 않는다.
- **같은 원본 URL을 다시 입력하면 항상 새 번호를 발급한다.** 기존 번호를 돌려주려면 긴 URL에 UNIQUE 제약(해시 컬럼)을 걸고, 동시 입력 경쟁도 처리해야 한다. 그래서 단순한 규칙을 택했다 (ADR-0002).
- 정규형만 인정한다. 1 이상이고, 앞자리에 0이 없고, `i64` 범위 안이어야 한다. `/0`, `/0123`, 범위를 넘는 숫자, 없는 번호는 모두 404 안내 페이지로 응답한다. 그래서 링크 하나에 단축 주소는 정확히 하나다.

### 중복 방지

- 번호는 DB 시퀀스가 원자적으로 나눠 준다. 동시에 요청이 들어와도 충돌하지 않는다.
- `PRIMARY KEY` 제약이 같은 번호의 두 번째 저장을 한 번 더 막는다.
- `GENERATED ALWAYS`라서 번호를 직접 지정한 `INSERT`도 거부된다. 그래서 이미 쓰인 번호를 다른 URL에 다시 쓸 수 없다.

### 생성 후 불변 (세 겹, ADR-0002)

1. **API**: 생성 경로만 있다. `/api/links/{n}`에 대한 `PUT`/`PATCH`/`DELETE`는 링크 존재 여부와 상관없이 `405`로 명시적으로 거부한다. 화면에 버튼이 없는 것에 그치지 않고, 서버가 거부한다.
2. **저장소 코드**: `src/links.rs`의 공개 함수는 `insert`와 `find`뿐이다.
3. **DB**: `migrations/0002_links_immutable.sql`의 트리거가 SQL로 직접 시도한 변경도 막는다.
   - `BEFORE UPDATE OR DELETE` 행 트리거
   - `BEFORE TRUNCATE` 문장 트리거

수정, 삭제, 자동 만료 기능은 없다.

### 원본 URL 검증 (서버)

`url` 크레이트로 파싱한 뒤 아래 조건을 모두 통과해야 링크로 만든다. 저장할 때는 정규화한(직렬화한) 값을 쓰며, 경로·쿼리·프래그먼트는 보존된다.

- 앞뒤 공백을 제거한다.
- scheme이 `http` 또는 `https`여야 한다.
- host가 있어야 한다.
- 2048자 이하여야 한다.
- 서비스 자신의 host(`PUBLIC_BASE_URL`)가 아니어야 한다. 단축 주소를 다시 단축하는 것을 막는다.

## 로컬 실행

### 준비물

- Rust stable + `wasm32-unknown-unknown` 타깃. Windows는 MSVC Build Tools(C++)가 필요하다.
- Dioxus CLI 0.7.10: `cargo install dioxus-cli --version 0.7.10 --locked`
- Docker (로컬 Postgres)
- Node.js (Cloudflare `wrangler`)

### DB 준비

```bash
docker run -d --name short-url-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17
```

스키마는 서버가 시작할 때 `sqlx::migrate!`로 자동 적용된다. 따로 마이그레이션을 실행하지 않아도 된다.

### 환경변수

[.env.example](.env.example)에 이름만 있다. 실제 값은 셸 환경변수나 배포 환경에 넣는다.

| 변수 | 위치 | 설명 |
| --- | --- | --- |
| `DATABASE_URL` | 백엔드, 테스트 | Postgres 접속 주소. Railway에서는 `${{Postgres.DATABASE_URL}}` |
| `PUBLIC_BASE_URL` | 백엔드 | 서비스 공개 주소(Worker 주소, 끝에 `/` 없이). 단축 주소의 기준 |
| `IP`, `PORT` | 백엔드 | 바인드 주소. 이미지는 `IP=0.0.0.0`이고, Railway가 `PORT`를 주입한다 |
| `BACKEND_ORIGIN` | Worker | Railway 백엔드 공개 주소. `wrangler.jsonc`의 `vars`나 `--var`로 지정 |

### 실행

**1. 백엔드 + 화면 (개발 서버)**

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8080 dx serve --fullstack
```

**2. 배포와 같은 구성으로 확인 (release 서버 + 로컬 Worker)**

먼저 release로 빌드한다.

```bash
dx build --release --fullstack
```

빌드 결과 폴더로 이동해 서버를 띄운다.

```bash
cd target/dx/short_url/release/web && IP=127.0.0.1 PORT=8081 DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8787 ./server
```

다른 터미널에서, 저장소 루트를 기준으로 Worker를 띄운다.

```bash
npx wrangler dev --config cloudflare/wrangler.jsonc --port 8787 --var BACKEND_ORIGIN:http://127.0.0.1:8081
```

브라우저로 `http://localhost:8787`을 연다. Windows에서는 서버 파일이 `server.exe`다.

### 검증 명령

```bash
cargo fmt --all -- --check
cargo clippy --features server --all-targets -- -D warnings
cargo clippy --features web --target wasm32-unknown-unknown -- -D warnings
cargo test --features server
dx build --release --fullstack
npx wrangler deploy --dry-run --config cloudflare/wrangler.jsonc
```

`cargo test`에는 `DATABASE_URL`이 필요하다. `#[sqlx::test]`가 테스트마다 격리된 DB를 만들고 마이그레이션을 적용한다.

## 배포

운영 비용은 0원이다 (ADR-0003). Railway에는 카드를 등록하지 않고 Trial/Free 크레딧 안에서만 운영한다. 크레딧이 떨어지면 서비스가 멈출 뿐 청구되지는 않는다. Cloudflare는 무료 플랜과 무료 `workers.dev` 주소를 쓴다.

### 1. Railway (백엔드 + DB)

1. Railway에 GitHub 계정으로 가입한다. 카드는 등록하지 않는다.
2. **New Project → Deploy from GitHub repo**에서 이 저장소를 고른다. 루트의 `railway.json` 때문에 `Dockerfile` 빌드와 `/api/health` 헬스 체크가 적용된다.
3. **+ New → Database → PostgreSQL**을 추가한다.
4. 백엔드 서비스 **Variables**에 다음을 넣는다.
   - `DATABASE_URL=${{Postgres.DATABASE_URL}}`
   - `PUBLIC_BASE_URL=<Worker 주소>` (3단계 뒤에 넣는다)
5. **Settings → Networking → Generate Domain**으로 `https://<백엔드>.up.railway.app`을 만든다.
6. **Settings → Deploy → Serverless**를 켠다. 유휴 시간에 절전해 크레딧을 아낀다. Postgres는 켜지 않는다.

`Dockerfile`은 멀티스테이지다. 1단계에서 `dx build --release --fullstack`으로 서버와 웹 번들을 빌드하고, 2단계에서 실행 파일과 `public/`만 `debian:trixie-slim`에 담는다.

### 2. Cloudflare (공개 접점)

출력 폴더를 지우고 깨끗하게 다시 빌드한다.

```bash
rm -rf target/dx/short_url/release/web && dx build --release --fullstack
```

Cloudflare에 로그인한다.

```bash
npx wrangler login
```

백엔드 주소를 넣어 배포한다.

```bash
npx wrangler deploy --config cloudflare/wrangler.jsonc
```

`wrangler.jsonc`의 `vars.BACKEND_ORIGIN`에 지금 쓰는 백엔드 주소(`https://shorturl-production-fb10.up.railway.app`)가 들어 있다. 공개 주소라 비밀 값이 아니다. 다른 백엔드를 가리키려면 `--var BACKEND_ORIGIN:https://<백엔드>.up.railway.app`을 붙인다.

배포가 끝나면 `https://short-url.<계정>.workers.dev`가 출력된다. 이 주소를 Railway의 `PUBLIC_BASE_URL`에 넣는다.

### 라우팅 구성

`cloudflare/src/worker.js`는 경로를 이렇게 나눈다.

- `/api` 또는 `/api/`로 시작하는 경로, 그리고 숫자로만 된 단일 세그먼트(`/^\/[0-9]+$/`): Railway 백엔드로 프록시한다.
  - `fetch(..., { redirect: "manual" })`로 받아, 백엔드의 301 응답과 `Location` 헤더를 그대로 브라우저에 전달한다.
  - 정규형 판정(`/0`, `/0123` → 404)은 백엔드가 한다.
  - Railway 절전에서 깨어나는 첫 GET이 502/503이면 한 번 다시 시도한다.
- 그 밖의 경로: `env.ASSETS.fetch`로 웹 화면 정적 파일을 준다.
- `not_found_handling`을 SPA로 두지 않는다. 그러면 브라우저에서 연 `/123`이 Worker 스크립트를 거치지 않기 때문이다.

## 확인 시나리오

| 시나리오 | 자동 테스트 | 재현 방법 |
| --- | --- | --- |
| 시크릿 창에서 생성하고 복사 | – (화면) | 시크릿 창에서 서비스 URL을 열고 URL 입력 → 생성 → 복사 |
| 다른 브라우저에서 단축 주소로 이동 | `tests/issue_and_redirect.rs` | 복사한 주소를 다른 브라우저에서 열어 원본으로 가는지 확인 |
| 서로 다른 URL 동시 생성 시 충돌·덮어쓰기 없음 | `tests/concurrency_persistence.rs` (50건 병렬) | 아래 동시 요청 명령 실행 후, 번호가 모두 다른지 확인 |
| 재시작·재배포 후에도 이전 주소로 이동 | `tests/concurrency_persistence.rs` (풀 재생성) | Railway에서 Redeploy 후 이전 단축 주소 열기 |
| 잘못된 URL | `tests/url_validation.rs` | `ftp://x`, `javascript:alert(1)`, 빈 값 입력 → 오류 메시지 |
| 존재하지 않는 숫자 | `tests/not_found.rs` | `/999999999`, `/0123` 열기 → 404 안내 |
| 기존 연결 변경 요청 | `tests/immutability_api.rs`, `tests/immutability_db.rs` | 아래 변경 요청 명령 → `405`, 이후 이동 결과 그대로 |

동시 요청 (bash):

```bash
for i in $(seq 1 20); do curl -s -X POST https://short-url.tbafe.workers.dev/api/links -H 'content-type: application/json' -d "{\"url\":\"https://example.com/$i\"}" & done; wait
```

변경 요청:

```bash
curl -i -X PUT https://short-url.tbafe.workers.dev/api/links/1 -H 'content-type: application/json' -d '{"url":"https://evil.example"}'
```

### 확인 결과

- **자동 테스트**: `cargo test --features server` 19개가 모두 통과한다. 이 중 DB 통합 테스트는 로컬 Postgres 17로 돌렸다.
- **로컬 종단 확인**: release 서버 앞에 `wrangler dev`를 두고 확인했다.
  - 생성 `201`
  - `/1` → `301 Location: https://example.org/path?q=1#frag`
  - `/0123`, `/999999` → `404`
  - `DELETE` → `405`
  - `ftp://` → `400`
  - 브라우저로 화면 표시, 생성, 이동(경로·쿼리·프래그먼트 보존), 오류 메시지, 연속 생성을 확인했다.
- **배포 환경 확인** (`https://short-url.tbafe.workers.dev`, 2026-11-05)
  - `/api/health` → `200 ok`, `/` → 웹 화면
  - 생성 `201` → `/2` → `301 Location: https://www.rust-lang.org/learn/get-started?lang=ko#installing-rust` (경로·쿼리·프래그먼트 보존)
  - `/999999999`, `/0123` → `404` 안내 페이지
  - `/api/links/2`에 `PUT`·`PATCH`·`DELETE` → `405`, 이후에도 `/2`의 이동 결과가 그대로
  - `javascript:alert(1)`, `ftp://x.org/a`, `not a url`, 서비스 자신의 주소 → `400`과 오류 메시지
  - 서로 다른 URL 20건 동시 생성 → 모두 `201`, 번호 20개가 모두 다르고 각각 자기 원본으로 이동
  - 브라우저에서 화면으로 생성 → 단축 주소 표시
  - Railway에서 백엔드를 재배포한 뒤 `/1`, `/2`, `/3`, `/23` → 이전과 같은 원본으로 `301`, 새 생성은 다음 번호(`24`)로 `201`
- **확인하지 않은 것**
  - 실제 클립보드 복사: 테스트 브라우저에 클립보드 권한이 없어서, 복사 함수에 단축 주소가 전달되는 것까지만 확인했다.
