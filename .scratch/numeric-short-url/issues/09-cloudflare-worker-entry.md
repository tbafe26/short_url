# 09: Cloudflare Worker 공개 접점 설정

**What to build:** Cloudflare Worker 하나가 Dioxus 웹 번들을 제공하고, `/api/*`와 숫자로만 된 단일 세그먼트 경로를 Railway 백엔드로 프록시한다. 브라우저는 같은 출처로만 요청한다. 사용자가 배포하면 `workers.dev` HTTPS 주소에서 시크릿 창으로 생성과 이동이 된다. 실제 배포(로그인 필요)는 사용자가 한다. 설계는 ADR-0001을 따른다.

**Blocked by:** 08

**Status:** resolved

- [x] `wrangler.jsonc`에 Static Assets 디렉터리, 프록시 스크립트, `BACKEND_ORIGIN` 변수가 있고 비밀 값은 없다
- [x] 프록시는 백엔드 응답(301 포함)을 `redirect: "manual"`로 받아 그대로 전달한다
- [x] 클라이언트의 API 호출이 같은 출처의 `/api/*`로 나간다 (서버 함수 대신 `POST /api/links`)
- [x] 웹 번들을 만들어 Worker assets 디렉터리에 넣는 빌드 절차가 있다
- [x] `npx wrangler deploy --dry-run`이 통과한다
- [x] 사용자가 따라 할 Cloudflare 배포 절차가 티켓 하단 `## Comments`에 적혀 있다

## Comments

### 로컬 종단 확인 (release 서버 + `wrangler dev`)

로컬 Postgres에 연결한 release `server.exe`(8081) 앞에 `wrangler dev`(8787, `BACKEND_ORIGIN=http://127.0.0.1:8081`)를 두고 확인했다.

- API를 Worker로 호출한 결과
  - 생성 201
  - `/1` → 301, `Location: https://example.org/path?q=1#frag`
  - `/0123`, `/999999` → 404
  - `DELETE /api/links/1` → 405
  - `ftp://` → 400
- 브라우저(앱 내장)로 `http://localhost:8787/`를 열어 확인한 결과
  - 화면과 탭 제목 "숫자형 단축 URL"이 뜬다.
  - 앞뒤에 공백을 넣은 URL로 생성하면 단축 주소가 표시되고, 그 주소로 이동하면 경로·쿼리·프래그먼트가 보존된 원본으로 간다.
  - 오류 메시지가 표시된다.
  - 새로고침 없이 연속으로 생성할 수 있다.

### 고친 문제: 정적 index.html에서 화면이 뜨지 않음

- 처음에는 `dioxus` 의존성에 `fullstack` 기능을 켜 두었다. 그 결과 웹 번들에 hydration 코드가 들어갔고, Worker의 정적 `index.html`(서버 렌더링 데이터 없음)에서 `atob` 오류로 빈 화면이 됐다.
- `Config::hydrate(false)`는 효과가 없었다. `hydrate` 기능이 컴파일되어 있으면 무조건 hydration을 시도하기 때문이다.
- 그래서 `dioxus/fullstack`을 `server` 기능으로만 옮겼다. 그러면 `dx`가 풀스택을 자동 감지하지 못하므로, 빌드 명령을 `dx build --release --fullstack`으로 바꿨다 (AGENTS.md, Dockerfile 반영).
- 탭 제목은 `document::Title`로 서비스 이름을 쓴다 (06의 남은 과제).

### Cloudflare 배포 절차 (사용자가 직접)

1. Railway 배포(티켓 08)를 먼저 끝내고 백엔드 주소 `https://<백엔드>.up.railway.app`를 확인한다.
2. 웹 번들을 깨끗하게 다시 빌드한다. 이전 빌드의 자산이 쌓이지 않도록 출력 폴더를 먼저 지운다.
   ```bash
   rm -rf target/dx/short_url/release/web && dx build --release --fullstack
   ```
3. Cloudflare에 로그인한다 (무료 계정, 카드 불필요). 브라우저가 열리면 승인한다.
   ```bash
   npx wrangler login
   ```
4. 백엔드 주소를 넣어 배포한다. 출력에 공개 주소 `https://short-url.<계정>.workers.dev`가 나온다.
   ```bash
   npx wrangler deploy --config cloudflare/wrangler.jsonc --var BACKEND_ORIGIN:https://<백엔드>.up.railway.app
   ```
   `cloudflare/wrangler.jsonc`의 `vars.BACKEND_ORIGIN`을 실제 주소로 바꿔 커밋해도 된다. 공개 주소라 비밀 값이 아니다.
5. Railway 백엔드의 `PUBLIC_BASE_URL`을 위 workers.dev 주소로 설정한다 (끝에 `/` 없이). 그러면 서비스가 다시 배포된다.
6. 시크릿 창으로 workers.dev 주소를 열어 생성, 복사, 이동을 확인한다.
