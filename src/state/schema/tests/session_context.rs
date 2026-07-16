use crate::state::Store;
use rusqlite::Connection;

#[test]
fn stamped_schema_without_session_execution_context_is_rejected() {
    for missing in ["work_root", "readiness_parent"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        drop(Store::open(&path).unwrap());
        let conn = Connection::open(&path).unwrap();
        conn.execute(&format!("ALTER TABLE sessions DROP COLUMN {missing}"), [])
            .unwrap();
        drop(conn);

        let error = Store::open(&path)
            .err()
            .expect("incomplete current-version schema must be rejected");
        let text = format!("{error:#}");
        assert!(text.contains("not the current canonical schema"), "{text}");
        assert!(text.contains(missing), "{text}");
    }
}
