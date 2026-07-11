use super::*;

#[test]
fn agent_label_preserves_backend_label() {
    assert_eq!(agent_label("codex", "myBackend"), "codex@myBackend");
    assert_eq!(agent_label("claude", "laptop"), "claude@laptop");
}

#[test]
fn session_handle_is_agent_dash_session_code() {
    assert_eq!(
        session_handle("codex", "willow-echo-042"),
        "codex-willow-echo-042"
    );
    assert_eq!(
        session_handle("codex", "codex-willow-echo-042"),
        "codex-willow-echo-042"
    );
    assert_eq!(session_handle("", "willow-echo-042"), "willow-echo-042");
}

#[test]
fn parses_session_handle() {
    assert_eq!(
        parse_session_handle("codex-willow-echo-042"),
        Some(("codex", "willow-echo-042"))
    );
    assert_eq!(
        parse_session_handle("chief-of-staff-willow-echo-042"),
        Some(("chief-of-staff", "willow-echo-042"))
    );
    assert_eq!(parse_session_handle("codex/echo123"), None);
    assert_eq!(parse_session_handle("codex"), None);
    assert_eq!(parse_session_handle("codex/"), None);
    assert_eq!(parse_session_handle("codex/echo/extra"), None);
    assert_eq!(parse_session_handle("chief-of-staff-smith"), None);
}

#[test]
fn profile_name_normalizes_backend_suffix() {
    assert_eq!(
        session_handle_from_profile_name("willow-echo-042@remoteBackend", "remoteBackend", "codex"),
        "codex-willow-echo-042"
    );
    assert_eq!(
        session_handle_from_profile_name("echo123@remoteBackend", "remoteBackend", ""),
        "echo123"
    );
}

#[test]
fn slug_from_profile_name_strips_matching_backend_suffix() {
    assert_eq!(
        slug_from_profile_name("developer1@remoteBackend", "remoteBackend"),
        "developer1"
    );
    assert_eq!(
        slug_from_profile_name("developer1", "remoteBackend"),
        "developer1"
    );
    assert_eq!(
        slug_from_profile_name("developer1@otherBackend", "remoteBackend"),
        "developer1@otherBackend"
    );
    assert_eq!(
        slug_from_profile_name("developer1@remoteBackend", ""),
        "developer1@remoteBackend"
    );
}

#[test]
fn agent_ref_from_is_bare_local_and_qualified_remote() {
    assert_eq!(agent_ref_from("developer", "laptop", "laptop"), "developer");
    assert_eq!(agent_ref_from("developer", "", "laptop"), "developer");
    assert_eq!(
        agent_ref_from("developer", "myBackend", "laptop"),
        "developer@myBackend"
    );
}

#[test]
fn session_label_preserves_session_handle() {
    assert_eq!(
        session_label("te-abc-0", "codex-willow-echo-042", "laptop"),
        "codex-willow-echo-042"
    );
    assert_eq!(session_label("te-abc-0", "codex", "laptop"), "codex@laptop");
    assert_eq!(session_label("te-abc-0", "", "laptop"), "te-abc-0");
}

#[test]
fn event_short_id_truncates_to_eight() {
    assert_eq!(event_short_id("0123456789abcdef"), "01234567");
    assert_eq!(event_short_id("abc"), "abc");
}

#[test]
fn parse_at_is_backend_label_not_channel() {
    match parse_ref("codex@myBackend") {
        Ref::Agent { slug, host } => {
            assert_eq!(slug, "codex");
            assert_eq!(host, "myBackend");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn agent_backend_ref_preserves_backend_label() {
    let r = parse_agent_backend_ref("claude@myBackend").unwrap();
    assert_eq!(r.slug, "claude");
    assert_eq!(r.backend.as_deref(), Some("myBackend"));

    let local = parse_agent_backend_ref("codex").unwrap();
    assert_eq!(local.slug, "codex");
    assert_eq!(local.backend, None);

    assert!(parse_agent_backend_ref("claude@").is_none());
    assert!(parse_agent_backend_ref("@laptop").is_none());
}

#[test]
fn parse_pubkey_and_token() {
    let hex = "a".repeat(64);
    assert!(matches!(parse_ref(&hex), Ref::Pubkey(_)));
    assert!(matches!(parse_ref("npub1abcdef"), Ref::Pubkey(_)));
    assert!(matches!(parse_ref("haiku1"), Ref::Token(_)));
    assert!(matches!(parse_ref("codex"), Ref::Token(_)));
}

#[test]
fn extract_inline_mentions() {
    assert_eq!(
        extract_mentions("hey @haiku-sable-grove-179 and @codex, look"),
        vec!["haiku-sable-grove-179".to_string(), "codex".to_string()]
    );
    assert_eq!(
        extract_mentions("ping @claude@tower please"),
        vec!["claude@tower".to_string()]
    );
    assert_eq!(
        extract_mentions("hey @codex-willow-echo-042 how are you?"),
        vec!["codex-willow-echo-042".to_string()]
    );
    assert_eq!(extract_mentions("ping @codex."), vec!["codex".to_string()]);
    assert!(extract_mentions("email dev@example.com please").is_empty());
    assert_eq!(
        extract_mentions("@haiku1 @haiku1"),
        vec!["haiku1".to_string()]
    );
}
