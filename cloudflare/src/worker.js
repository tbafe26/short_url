// 공개 접점 Worker (ADR-0001).
// 브라우저는 이 Worker의 출처로만 요청한다. 그래서 CORS가 필요 없고 백엔드 주소도 노출되지 않는다.

// 숫자로만 된 단일 세그먼트. 정규형 판정(`/0`, `/0123` 거부)은 백엔드가 한다.
const NUMERIC_PATH = /^\/[0-9]+$/;

/** 백엔드로 보낼 경로인지 판단한다. */
export function isBackendPath(pathname) {
  return pathname === "/api" || pathname.startsWith("/api/") || NUMERIC_PATH.test(pathname);
}

/** 요청을 백엔드로 그대로 전달한다. 이동(301) 응답은 따라가지 않고 그대로 돌려준다. */
async function proxy(request, backendOrigin) {
  const url = new URL(request.url);
  const target = new URL(url.pathname + url.search, backendOrigin);
  const send = () => fetch(new Request(target, request), { redirect: "manual" });

  const response = await send();
  // Railway 절전에서 깨어나는 첫 요청은 502/503일 수 있다. 본문이 없는 GET만 한 번 다시 시도한다.
  if ((response.status === 502 || response.status === 503) && request.method === "GET") {
    return send();
  }
  return response;
}

export default {
  async fetch(request, env) {
    const { pathname } = new URL(request.url);
    if (isBackendPath(pathname)) {
      return proxy(request, env.BACKEND_ORIGIN);
    }
    return env.ASSETS.fetch(request);
  },
};
