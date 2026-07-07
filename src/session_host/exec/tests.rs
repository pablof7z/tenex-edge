use super::*;
use std::io::Write as _;

#[test]
fn extracts_native_id_from_json_lines() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("headless.log");
    let mut file = std::fs::File::create(&log).unwrap();
    writeln!(file, "non-json warning").unwrap();
    writeln!(
        file,
        "{}",
        serde_json::json!({
            "type": "event",
            "payload": {
                "session": { "id": "native-session-1" }
            }
        })
    )
    .unwrap();

    assert_eq!(
        extract_native_session_id(&log).as_deref(),
        Some("native-session-1")
    );
}
