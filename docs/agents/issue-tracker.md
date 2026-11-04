# 이슈 트래커: 로컬 마크다운

이 저장소의 이슈와 스펙은 `.scratch/` 아래 마크다운 파일로 관리한다.

## 규칙

- 기능 하나당 디렉터리 하나: `.scratch/<feature-slug>/`
- 스펙은 `.scratch/<feature-slug>/spec.md`
- 구현 이슈는 티켓 하나당 파일 하나: `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, `01`부터 번호를 매긴다. 여러 티켓을 한 파일에 합치지 않는다.
- 트리아지 상태는 각 이슈 파일 상단의 `Status:` 줄에 기록한다 (역할 문자열은 `triage-labels.md` 참고)
- 코멘트와 대화 기록은 파일 하단의 `## Comments` 헤딩 아래에 이어 붙인다

## 스킬이 "이슈 트래커에 게시"하라고 할 때

`.scratch/<feature-slug>/` 아래에 새 파일을 만든다 (디렉터리가 없으면 생성).

## 스킬이 "관련 티켓을 가져오라"고 할 때

참조된 경로의 파일을 읽는다. 보통 사용자가 경로나 이슈 번호를 직접 넘겨준다.

## Wayfinding 작업

`/wayfinder`에서 사용한다. **map**은 파일 하나이고, 티켓마다 **child** 파일이 하나씩 있다.

- **Map**: `.scratch/<effort>/map.md` (Notes / Decisions-so-far / Fog 본문).
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, `01`부터 번호를 매기며 본문에 질문을 적는다. `Type:` 줄에 티켓 유형(`research`/`prototype`/`grilling`/`task`)을, `Status:` 줄에 `claimed`/`resolved`를 기록한다.
- **Blocking**: 상단의 `Blocked by: NN, NN` 줄. 나열된 파일이 모두 `resolved`이면 차단이 풀린다.
- **Frontier**: `.scratch/<effort>/issues/`에서 열려 있고, 차단되지 않았고, 아무도 claim하지 않은 파일을 찾는다. 번호가 가장 낮은 것이 우선.
- **Claim**: 작업 전에 `Status: claimed`로 바꾸고 저장한다.
- **Resolve**: `## Answer` 헤딩 아래 답을 붙이고 `Status: resolved`로 바꾼 뒤, `map.md`의 Decisions-so-far에 컨텍스트 포인터(요지 + 링크)를 추가한다.
