# 09: Cloudflare Worker 공개 접점 설정

**What to build:** Cloudflare Worker 하나가 Dioxus 웹 번들을 제공하고, `/api/*`와 숫자로만 된 단일 세그먼트 경로를 Railway 백엔드로 프록시한다. 브라우저는 같은 출처로만 요청한다. 사용자가 배포하면 `workers.dev` HTTPS 주소에서 시크릿 창으로 생성과 이동이 된다. 실제 배포(로그인 필요)는 사용자가 한다. 설계는 ADR-0001을 따른다.

**Blocked by:** 08

**Status:** ready-for-agent

- [ ] `wrangler.jsonc`에 Static Assets 디렉터리, 프록시 스크립트, `BACKEND_ORIGIN` 변수가 있고 비밀 값은 없다
- [ ] 프록시는 백엔드 응답(301 포함)을 `redirect: "manual"`로 받아 그대로 전달한다
- [ ] 클라이언트의 서버 함수 호출이 같은 출처의 `/api/*`로 나간다
- [ ] 웹 번들을 만들어 Worker assets 디렉터리에 넣는 빌드 절차가 있다
- [ ] `npx wrangler deploy --dry-run`이 통과한다
- [ ] 사용자가 따라 할 Cloudflare 배포 절차가 티켓 하단 `## Comments`에 적혀 있다
