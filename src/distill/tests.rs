use super::*;

#[test]
fn command_distiller_uses_stdout_first_line() {
    let d = CommandDistiller {
        command: "cat >/dev/null; printf 'Fixing the auth bug\\nignored second line'".into(),
    };
    assert_eq!(
        d.summarize("User: fix the auth bug").unwrap(),
        "Fixing the auth bug"
    );
}

#[test]
fn command_distiller_none_on_failure() {
    let d = CommandDistiller {
        command: "exit 1".into(),
    };
    assert!(d.summarize("anything").is_none());
}

#[test]
fn command_distiller_none_on_empty_output() {
    let d = CommandDistiller {
        command: "cat >/dev/null; true".into(),
    };
    assert!(d.summarize("anything").is_none());
}

#[test]
fn parse_labels_reads_both_lines() {
    let (title, activity) =
        parse_labels("TITLE: Fix GitHub issue 1\nNOW: reading the issue tracker");
    assert_eq!(title.as_deref(), Some("Fix GitHub issue 1"));
    assert_eq!(activity.as_deref(), Some("reading the issue tracker"));
}

#[test]
fn parse_labels_is_case_and_synonym_tolerant() {
    let (title, activity) = parse_labels("title:  Refactor parser  \nActivity: writing tests.");
    assert_eq!(title.as_deref(), Some("Refactor parser"));
    assert_eq!(activity.as_deref(), Some("writing tests"));
}

#[test]
fn parse_labels_bare_line_is_title() {
    let (title, activity) = parse_labels("Fixing the auth bug");
    assert_eq!(title.as_deref(), Some("Fixing the auth bug"));
    assert_eq!(activity, None);
}

/// Drive `distill_session` through the external-command seam. Both scenarios
/// live in one test: `TENEX_EDGE_DISTILL_CMD` is process-global.
#[tokio::test]
async fn distill_session_via_command() {
    let mut env = crate::test_env::EnvGuard::set(
        "TENEX_EDGE_DISTILL_CMD",
        "cat >/dev/null; printf 'TITLE: Fix GitHub issue 1\\nNOW: reading the issue tracker\\n'",
    );
    let (got, err) = distill_session("user: fix github issue 1", None, "test-session").await;
    assert!(err.is_none());
    let got = got.unwrap();
    assert_eq!(got.title, "Fix GitHub issue 1");
    assert_eq!(got.activity, "reading the issue tracker");

    env.set_var(
        "TENEX_EDGE_DISTILL_CMD",
        "sed -n 's/^CURRENT TITLE: /TITLE: /p' | head -n1",
    );
    let (got, err) = distill_session(
        "TRANSCRIPT:\nuser: keep going",
        Some("refactoring the auth flow"),
        "test-session",
    )
    .await;
    assert!(err.is_none());
    let got = got.unwrap();
    assert_eq!(got.title, "refactoring the auth flow");
    assert_eq!(got.activity, "");
}

/// Empty transcript returns the current title rather than re-distilling.
#[tokio::test]
async fn distill_session_empty_transcript_returns_current() {
    let (got, err) = distill_session("   ", Some("writing the parser"), "test-session").await;
    assert!(err.is_none());
    let got = got.unwrap();
    assert_eq!(got.title, "writing the parser");
    assert_eq!(got.activity, "");
}
