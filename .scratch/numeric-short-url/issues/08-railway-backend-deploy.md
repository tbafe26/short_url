# 08: Railway 백엔드 배포 설정

**What to build:** 백엔드를 Railway에 올릴 수 있는 설정과 절차를 만든다. 사용자가 절차대로 배포하면 Railway 공개 주소에서 생성 API와 `/{n}` 이동이 동작한다. 실제 배포(로그인 필요)는 사용자가 한다. 설계는 ADR-0003과 스펙의 "배포"를 따른다.

**Blocked by:** 02

**Status:** resolved

- [x] 멀티스테이지 `Dockerfile`(최종 단계는 실행 파일과 `public/`만)이 있다. 디스크 여유가 부족해 로컬 `docker build`는 하지 않고, 정적 검토(베이스 이미지·단계별 COPY 경로·`dx` 산출물 위치·`IP`/`PORT`)로 확인한다. 실제 빌드는 Railway 첫 배포에서 확인한다
- [x] Railway 설정 파일에 Dockerfile 빌드와 헬스 체크 경로(`/api/health`)가 지정되어 있고, 헬스 체크 엔드포인트가 구현되어 있다
- [x] 서버가 Railway가 주입하는 `PORT`로 `0.0.0.0`에 바인드한다
- [x] 비밀 값이 저장소에 없다
- [x] 사용자가 따라 할 Railway 배포 절차(Postgres 추가, 환경변수, 절전 켜기, 카드 미등록)가 티켓 하단 `## Comments`에 적혀 있다

## Comments

### 구현 메모

- cargo-chef는 쓰지 않았다. `dx`가 자체 프로필(`server-release`, `wasm-release`)로 빌드해서 cargo-chef가 미리 빌드한 의존성을 재사용하지 못하기 때문이다. 그 대신 `dx`는 cargo-binstall로 미리 빌드된 바이너리를 받아 빌드 시간을 줄였다.
- 정적 검토 근거: 로컬 `dx build --release`의 산출물이 `target/dx/short_url/release/web/` 아래 `server`(Windows는 `server.exe`)와 `public/`로 나온다. 그래서 이 폴더를 통째로 `/app/`에 복사하고 `/app/server`를 실행한다. 서버는 `ENV IP=0.0.0.0`과 Railway가 주입하는 `PORT`로 바인드한다.
- 런타임 이미지는 `debian:trixie-slim`과 `ca-certificates`다. TLS는 rustls라 OpenSSL이 필요 없다.

### Railway 배포 절차 (사용자가 직접)

1. https://railway.com 에 GitHub 계정으로 가입한다. **카드를 등록하지 않는다** (Trial/Free 크레딧 안에서만 운영, ADR-0003).
2. **New Project → Deploy from GitHub repo**에서 이 저장소를 고른다. 배포 브랜치는 제출 브랜치다. 루트의 `railway.json` 때문에 Dockerfile 빌드와 `/api/health` 헬스 체크가 자동 적용된다.
3. 같은 프로젝트에서 **+ New → Database → PostgreSQL**을 추가한다.
4. 백엔드 서비스의 **Variables**에 다음을 넣는다.
   - `DATABASE_URL` = `${{Postgres.DATABASE_URL}}` (Railway 참조 변수, 비공개 내부망 주소)
   - `PUBLIC_BASE_URL` = Cloudflare Worker 공개 주소 (예: `https://short-url.<계정>.workers.dev`, 티켓 09에서 정해진다. 끝에 `/` 없이)
   - `PORT`는 Railway가 주입하므로 넣지 않는다.
5. 백엔드 서비스 **Settings → Networking → Generate Domain**으로 `*.up.railway.app` 주소를 만든다. 이 주소가 Worker의 `BACKEND_ORIGIN`이 된다.
6. 백엔드 서비스 **Settings → Deploy → Serverless**를 켠다 (절전). Postgres 서비스는 켜지 않는다.
7. 배포 로그에서 빌드 성공을 확인하고, `https://<백엔드>.up.railway.app/api/health`가 `ok`인지 확인한다. 빌드가 메모리나 시간 부족으로 실패하면 로그를 공유한다. 이때는 GitHub Actions 빌드로 바꾸는 대안이 있다 (ADR-0003).
8. 서버는 시작할 때 마이그레이션을 자동으로 적용한다. DB를 손으로 준비할 필요가 없다.
