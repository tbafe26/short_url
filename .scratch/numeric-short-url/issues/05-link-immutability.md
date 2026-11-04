# 05: 링크 불변성 (405 + DB 트리거)

**What to build:** 누구도 발급된 링크를 수정·삭제할 수 없다. API는 변경 요청을 `405`로 명시적으로 거부하고, DB는 직접 SQL로 시도한 변경도 트리거로 거부한다. 설계는 ADR-0002와 스펙의 "스키마"·"API 계약"을 따른다.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `PUT`/`PATCH`/`DELETE /api/links/{n}`이 링크 존재 여부와 상관없이 `405`, `Allow` 헤더, 불변성 설명 본문을 응답하고, 이후 이동 결과는 그대로다 (경계 1 테스트)
- [ ] `BEFORE UPDATE OR DELETE` 행 트리거와 `BEFORE TRUNCATE` 문장 트리거가 마이그레이션으로 추가된다
- [ ] 직접 SQL `UPDATE`·`DELETE`·`TRUNCATE`와 번호를 지정한 `INSERT`가 모두 실패하고 데이터가 그대로다 (경계 2 테스트)
