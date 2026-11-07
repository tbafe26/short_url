# 숫자형 단축 URL

로그인 없이 긴 URL을 `/123`처럼 숫자로만 된 단축 주소로 바꾸는 Dioxus 풀스택(Rust) 서비스입니다.

<br>

## 화면

<p align="center">
  <img src="docs/images/result.png" alt="단축 주소를 만들고 복사한 뒤의 화면" width="520">
</p>

<br>

## 주요 기능

- **숫자 주소 발급**: 원본 URL을 넣으면 `/123`처럼 숫자로만 된 단축 주소를 만들고, 버튼 하나로 복사합니다. 같은 원본 URL도 넣을 때마다 새 번호를 받습니다.
- **그대로 이동**: 단축 주소로 들어오면 `301`로 원본 URL에 보내고, 경로·쿼리·프래그먼트를 그대로 보존합니다.
- **입력 검증과 안내**: 서버가 원본 URL을 검증해(`http`/`https`, host 필수, 2048자 이하, 서비스 자신의 주소 거부) 실패하면 안내 메시지를 보여 주고, 없는 번호와 `/0`, `/0123` 같은 비정규형 숫자에는 404 안내 페이지를 보여 줍니다.
- **바뀌지 않는 링크**: 한 번 만든 링크는 수정·삭제할 수 없습니다. API는 변경 요청을 `405`로 거부하고, 저장소 코드에는 `insert`와 `find`만 있으며, DB 트리거가 `UPDATE`·`DELETE`·`TRUNCATE`를 막습니다.

<br>

## 기술 스택

| 영역 | 기술 |
| --- | --- |
| 프런트엔드 | ![Dioxus](https://img.shields.io/badge/Dioxus_0.7-00A8D6?logoColor=white) ![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?logo=webassembly&logoColor=white) |
| 백엔드 | ![Rust](https://img.shields.io/badge/Rust_2024-000000?logo=rust&logoColor=white) ![Axum](https://img.shields.io/badge/Axum-000000?logoColor=white) ![Tokio](https://img.shields.io/badge/Tokio-000000?logoColor=white) ![SQLx](https://img.shields.io/badge/SQLx-000000?logoColor=white) |
| DB | ![PostgreSQL](https://img.shields.io/badge/PostgreSQL-4169E1?logo=postgresql&logoColor=white) |
| 배포 | ![Cloudflare Workers](https://img.shields.io/badge/Cloudflare_Workers-F38020?logo=cloudflareworkers&logoColor=white) ![Railway](https://img.shields.io/badge/Railway-0B0D0E?logo=railway&logoColor=white) ![Docker](https://img.shields.io/badge/Docker-2496ED?logo=docker&logoColor=white) |

<br>

## 구성

<p align="center">
  <img src="docs/images/architecture.svg" alt="사용자 → Cloudflare Worker → Railway 백엔드 → PostgreSQL" width="100%">
</p>

| 구성 | 역할 |
| --- | --- |
| Cloudflare Worker | 공개 주소. `/api/*`와 `/{숫자}`는 백엔드로 넘기고(301 응답은 그대로 전달), 나머지는 웹 화면을 제공합니다. |
| Railway 백엔드 | 생성 API, 301 이동, 404 안내, 변경 거부. `Dockerfile`로 빌드합니다. |
| Railway PostgreSQL | 링크 저장. 단축 번호는 `BIGINT GENERATED ALWAYS AS IDENTITY` 기본키로 DB만 발급해, 동시에 요청이 들어와도 번호가 겹치지 않습니다. |

<br>

## 실행 방법

필요 도구: Rust(`wasm32-unknown-unknown` 타깃), Dioxus CLI 0.7.10, Docker

로컬 Postgres를 띄웁니다.

```bash
docker run -d --name short-url-pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17
```

서버와 화면을 띄웁니다. 스키마는 서버가 시작할 때 자동으로 적용됩니다.

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/short_url PUBLIC_BASE_URL=http://localhost:8080 dx serve --fullstack
```

테스트도 같은 `DATABASE_URL`로 실행합니다.

```bash
cargo test --features server
```

### 환경변수

[`.env.example`](.env.example)에 이름만 있습니다. 실제 값은 `.env`(저장소 제외)나 배포 환경에만 넣습니다.

| 변수 | 쓰는 곳 | 설명 |
| --- | --- | --- |
| `DATABASE_URL` | 백엔드 | Postgres 접속 주소(필수) |
| `PUBLIC_BASE_URL` | 백엔드 | 서비스 공개 주소. 단축 주소의 기준(필수) |
| `IP`, `PORT` | 백엔드 | 바인드 주소와 포트 |
| `BACKEND_ORIGIN` | Worker | 백엔드 공개 주소 |
