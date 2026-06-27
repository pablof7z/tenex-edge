use super::registry::{build_resume_command, resume_shape_for_bin, ResumeShape};

fn cmd(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn append_flag_preserves_user_launch_flags() {
    let base = cmd(&["claude", "--dangerously-skip-permissions"]);
    let got = build_resume_command(&base, ResumeShape::AppendFlag("--resume"), "abc-123");
    assert_eq!(
        got,
        cmd(&[
            "claude",
            "--dangerously-skip-permissions",
            "--resume",
            "abc-123"
        ])
    );
}

#[test]
fn append_flag_bare_command() {
    let got = build_resume_command(
        &cmd(&["opencode"]),
        ResumeShape::AppendFlag("--session"),
        "ses_x",
    );
    assert_eq!(got, cmd(&["opencode", "--session", "ses_x"]));
}

#[test]
fn subcommand_inserts_after_binary_and_keeps_flags() {
    let base = cmd(&["codex", "--dangerously-bypass-approvals-and-sandbox"]);
    let got = build_resume_command(&base, ResumeShape::Subcommand("resume"), "uuid-9");
    assert_eq!(
        got,
        cmd(&[
            "codex",
            "resume",
            "uuid-9",
            "--dangerously-bypass-approvals-and-sandbox"
        ])
    );
}

#[test]
fn subcommand_bare_command() {
    let got = build_resume_command(
        &cmd(&["codex"]),
        ResumeShape::Subcommand("resume"),
        "uuid-9",
    );
    assert_eq!(got, cmd(&["codex", "resume", "uuid-9"]));
}

#[test]
fn shape_is_keyed_by_binary_not_slug() {
    assert!(matches!(
        resume_shape_for_bin("claude"),
        Some(ResumeShape::AppendFlag("--resume"))
    ));
    assert!(matches!(
        resume_shape_for_bin("codex"),
        Some(ResumeShape::Subcommand("resume"))
    ));
    assert!(matches!(
        resume_shape_for_bin("opencode"),
        Some(ResumeShape::AppendFlag("--session"))
    ));
    assert!(matches!(
        resume_shape_for_bin("/opt/homebrew/bin/claude"),
        Some(ResumeShape::AppendFlag("--resume"))
    ));
    assert!(resume_shape_for_bin("npx").is_none());
}

fn sample_session() -> crate::state::SessionRecord {
    crate::state::SessionRecord {
        session_id: "sess-target".into(),
        agent_slug: "claude".into(),
        agent_pubkey: "pk-target".into(),
        project: "proj".into(),
        host: "host-a".into(),
        child_pid: None,
        watch_pid: None,
        created_at: 1000,
        alive: true,
        rel_cwd: String::new(),
        channel: String::new(),
    }
}

#[test]
fn pending_message_prompt_contains_the_actual_message_body() {
    let rec = sample_session();
    let row = crate::state::ChatInboxRow {
        chat_event_id: "abcdef123456".into(),
        target_session: rec.session_id.clone(),
        from_pubkey: "pk-sender".into(),
        from_slug: "codex".into(),
        project: "proj".into(),
        body: "please review the tmux delivery path".into(),
        created_at: 100,
        from_session: "sender-session".into(),
        mentioned_session: rec.session_id.clone(),
    };

    let prompt = crate::injection::render_direct_mention_prompt(&[row], 120).unwrap();

    assert!(prompt.contains("Incoming message mentioning this agent"));
    assert!(prompt.contains("Mention in #proj from codex"));
    assert!(prompt.contains("please review the tmux delivery path"));
    assert!(!prompt.contains("tenex-edge inbox"));
    assert!(!prompt.contains("project chat - write"));
}
