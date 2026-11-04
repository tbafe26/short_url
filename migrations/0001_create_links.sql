-- 링크 테이블. 단축 번호는 identity로만 발급한다(애플리케이션이 지정하지 않는다).
-- 같은 원본 URL에 UNIQUE를 두지 않는다: 매번 새 링크를 발급한다.
CREATE TABLE links (
    number       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    original_url TEXT NOT NULL CHECK (char_length(original_url) <= 2048),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
