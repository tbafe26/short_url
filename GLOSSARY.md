# 숫자형 단축 URL

누구나 로그인 없이 긴 URL을 숫자로만 된 짧은 주소로 바꾸고, 그 주소로 접속하면 원래 URL로 이동하는 서비스.

## Language

**원본 URL (Original URL)**:
사용자가 단축하려고 입력한 `http://` 또는 `https://` 주소.
_Avoid_: 긴 URL, 대상 URL, target

**단축 번호 (Short Number)**:
링크 하나를 식별하는, 숫자로만 이루어진 고유한 값. 한 번 발급되면 다른 원본 URL에 다시 쓰이지 않는다.
_Avoid_: 코드, slug, ID, 별칭

**단축 주소 (Short URL)**:
서비스 도메인 뒤에 단축 번호를 붙인 공개 주소. 예: `https://short.example/123`.
_Avoid_: 짧은 링크, short link

**링크 (Link)**:
단축 번호와 원본 URL의 불변 연결 한 건. 생성만 가능하고 수정·삭제·만료되지 않는다.
_Avoid_: 매핑, 레코드, 엔트리

**발급 (Issue)**:
새 링크를 만들면서 아직 쓰인 적 없는 단축 번호를 배정하는 일.
_Avoid_: 할당, 생성(번호에 대해)

**이동 (Redirect)**:
단축 주소 접속을 HTTP 리디렉션으로 원본 URL에 보내는 일.
_Avoid_: 포워딩, 리다이렉트 (표기 혼용 금지)
