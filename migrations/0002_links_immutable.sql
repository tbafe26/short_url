-- 링크는 불변이다 (ADR-0002). SQL을 직접 실행해도 수정·삭제·비우기를 할 수 없게 막는다.
-- 번호를 지정한 INSERT는 0001의 GENERATED ALWAYS가 이미 거부한다.

CREATE FUNCTION reject_link_change() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION '링크는 변경할 수 없다 (%)', TG_OP
        USING ERRCODE = 'restrict_violation';
END;
$$;

-- 행 단위: UPDATE, DELETE
CREATE TRIGGER links_reject_update_delete
    BEFORE UPDATE OR DELETE ON links
    FOR EACH ROW EXECUTE FUNCTION reject_link_change();

-- 문장 단위: TRUNCATE는 행 트리거를 거치지 않으므로 따로 막는다
CREATE TRIGGER links_reject_truncate
    BEFORE TRUNCATE ON links
    FOR EACH STATEMENT EXECUTE FUNCTION reject_link_change();
