use crate::cli::messaging::{format_envelope, EnvelopeView};

fn view<'a>() -> EnvelopeView<'a> {
    EnvelopeView {
        from_slug: "amber-codex",
        from_session: "sender-session-id",
        host: "",
        self_host: "my-box",
        subject: "NIP-29 group creation failing",
        branch: "features/oauth",
        commit: "a1b2c3d",
        dirty: 0,
        id: "01234567",
        sent_at: 1_000,
        now: 1_180,
        body: "can you take a look?",
    }
}

#[test]
fn envelope_has_email_like_headers_then_body() {
    let out = format_envelope(&view());
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "From: amber-codex");
    assert!(lines[1].starts_with("Date: ") && lines[1].ends_with("(3 min ago)"));
    assert_eq!(lines[2], "Subject: NIP-29 group creation failing");
    assert_eq!(lines[3], "Branch: features/oauth (a1b2c3d)");
    assert_eq!(lines[4], "ID: 01234567");
    assert_eq!(lines[5], "--");
    assert_eq!(lines[6], "can you take a look?");
}

#[test]
fn dirty_count_and_remote_host_annotate() {
    let mut view = view();
    view.dirty = 1;
    view.host = "prodBackend";
    let out = format_envelope(&view);
    assert!(out.contains("From: amber-codex"));
    assert!(out.contains("Branch: features/oauth (a1b2c3d) [1 file dirty]"));
    view.dirty = 3;
    assert!(format_envelope(&view).contains("[3 files dirty]"));
}

#[test]
fn subject_and_branch_lines_omitted_when_empty() {
    let mut view = view();
    view.subject = "";
    view.branch = "";
    let out = format_envelope(&view);
    assert!(!out.contains("Subject:"));
    assert!(!out.contains("Branch:"));
    assert!(!out.contains("remote:"));
}
