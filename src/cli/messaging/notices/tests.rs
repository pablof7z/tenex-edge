use super::*;

#[test]
fn renderer_preserves_daemon_owned_reminders() {
    let result = serde_json::json!({
        "recipient_reminders": [
            "Reminder: @one is suspended and will receive this message after manual resumption.",
            "Reminder: @two is suspended and will receive this message after manual resumption."
        ]
    });

    assert_eq!(
        recipient_reminders(&result).unwrap(),
        vec![
            "Reminder: @one is suspended and will receive this message after manual resumption.",
            "Reminder: @two is suspended and will receive this message after manual resumption."
        ]
    );
}

#[test]
fn renderer_rejects_an_incomplete_result_contract() {
    let error = recipient_reminders(&serde_json::json!({})).unwrap_err();
    assert!(error
        .to_string()
        .contains("daemon response missing recipient_reminders"));
}
