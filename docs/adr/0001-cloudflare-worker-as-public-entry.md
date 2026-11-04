# Cloudflare Worker 하나를 공개 접점으로 두고 Railway로 프록시한다

과제는 웹 화면을 Cloudflare에, 백엔드를 Railway에 두면서 공개 주소의 `/숫자`도 실제로 이동하도록 요구한다. 그래서 Workers(Static Assets) 하나가 Dioxus 웹 번들을 직접 제공하고, `/api/*`와 `/^[1-9][0-9]*$/` 경로는 Railway로 프록시한다. 이동 응답은 `redirect: "manual"`로 그대로 통과시킨다. 브라우저는 같은 출처만 호출하므로 CORS 설정이 필요 없고, Railway 주소는 Worker 환경변수에만 둔다.

## Considered Options

- **Cloudflare DNS 프록시로 Railway가 모든 경로 처리**: 웹 화면이 Cloudflare에 배포되지 않아 요구사항을 어긴다.
- **Cloudflare Pages + `_redirects`**: `_redirects`는 외부 출처로 프록시하지 못한다. Pages Functions를 더해야 하므로 Worker 하나보다 구성 요소가 많다.
