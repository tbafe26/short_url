# 08: Railway 백엔드 배포 설정

**What to build:** 백엔드를 Railway에 올릴 수 있는 설정과 절차를 만든다. 사용자가 절차대로 배포하면 Railway 공개 주소에서 생성 API와 `/{n}` 이동이 동작한다. 실제 배포(로그인 필요)는 사용자가 한다. 설계는 ADR-0003과 스펙의 "배포"를 따른다.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] 멀티스테이지 `Dockerfile`(cargo-chef 캐시, 최종 단계는 실행 파일과 필요한 자산만)이 있고 로컬에서 `docker build`가 성공한다
- [ ] Railway 설정 파일에 Dockerfile 빌드와 헬스 체크 경로(`/api/health`)가 지정되어 있고, 헬스 체크 엔드포인트가 구현되어 있다
- [ ] 서버가 Railway가 주입하는 `PORT`로 `0.0.0.0`에 바인드한다
- [ ] 비밀 값이 저장소에 없다
- [ ] 사용자가 따라 할 Railway 배포 절차(Postgres 추가, 환경변수, 절전 켜기, 카드 미등록)가 티켓 하단 `## Comments`에 적혀 있다
