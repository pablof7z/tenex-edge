use super::*;

#[path = "tests/live.rs"]
mod live;

#[test]
fn transport_kind_strings() {
    assert_eq!(TransportKind::Pty.as_str(), "pty");
    assert_eq!(TransportKind::Acp.as_str(), "acp");
}

#[test]
fn no_bundle_selects_pty() {
    // Agents without a configured harness bundle stay on the PTY transport.
    let cfg = crate::harness::config::HarnessesConfig::default();
    let t = select_transport_with(&cfg, None).unwrap();
    assert_eq!(t.kind(), TransportKind::Pty);
}

#[test]
fn builtin_slug_bundle_selects_pty() {
    // A bare harness slug (no harnesses.json entry) resolves to the PTY default.
    let cfg = crate::harness::config::HarnessesConfig::default();
    assert_eq!(
        transport_kind_for(&cfg, Some("claude")).unwrap(),
        TransportKind::Pty
    );
}

#[test]
fn acp_and_app_server_bundles_select_acp() {
    let json = r#"{
        "claude-acp": { "harness": "claude", "transport": "acp" },
        "codex-app": { "harness": "codex", "transport": "app-server" },
        "oc-pty":    { "harness": "opencode", "transport": "pty" },
        "cx-exec":   { "harness": "codex", "transport": "headless-exec" }
    }"#;
    let cfg: crate::harness::config::HarnessesConfig = serde_json::from_str(json).unwrap();
    assert_eq!(
        transport_kind_for(&cfg, Some("claude-acp")).unwrap(),
        TransportKind::Acp
    );
    assert_eq!(
        select_transport_with(&cfg, Some("codex-app"))
            .unwrap()
            .kind(),
        TransportKind::Acp
    );
    assert_eq!(
        transport_kind_for(&cfg, Some("oc-pty")).unwrap(),
        TransportKind::Pty
    );
    // Defect #5: strict resolution refuses to collapse headless-exec onto the PTY.
    assert!(transport_kind_for(&cfg, Some("cx-exec")).is_err());
}

/// Defect #5: a `headless-exec` bundle must not be silently collapsed onto the
/// interactive PTY. Launch selection hard-bails; the raw fail-open resolver does
/// NOT special-case it (the launch mapping owns the refusal, not the resolver).
#[test]
fn headless_exec_bundle_is_unsupported_at_launch() {
    let json = r#"{ "cx-exec": { "harness": "codex", "transport": "headless-exec" } }"#;
    let cfg: crate::harness::config::HarnessesConfig = serde_json::from_str(json).unwrap();
    // `TransportImpl` is not `Debug`, so match rather than `unwrap_err`.
    let err = match select_transport_with(&cfg, Some("cx-exec")) {
        Ok(_) => panic!("headless-exec bundle must not select a hosted-session transport"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("headless-exec"));
    assert_eq!(
        resolve_transport_fail_open_with("cx-exec", Ok(cfg)),
        crate::harness::Transport::HeadlessExec
    );
}

#[test]
fn unknown_bundle_fails_loud() {
    let cfg = crate::harness::config::HarnessesConfig::default();
    assert!(transport_kind_for(&cfg, Some("not-a-harness")).is_err());
}

// ── defect #3: launch-time transport resolution fails open to PTY ─────────────

#[test]
fn fail_open_resolves_acp_bundle_when_config_loads() {
    let json = r#"{ "claude-acp": { "harness": "claude", "transport": "acp" } }"#;
    let cfg: crate::harness::config::HarnessesConfig = serde_json::from_str(json).unwrap();
    assert_eq!(
        resolve_transport_fail_open_with("claude-acp", Ok(cfg)),
        crate::harness::Transport::Acp
    );
}

#[test]
fn fail_open_falls_back_to_pty_on_malformed_config() {
    // A malformed harnesses.json surfaces as an Err from the loader; the launch
    // path must NOT abort — it degrades to the PTY, not fail the launch.
    let load_err = Err(anyhow::anyhow!("parsing harnesses config: trailing comma"));
    assert_eq!(
        resolve_transport_fail_open_with("claude-acp", load_err),
        crate::harness::Transport::Pty
    );
}

#[test]
fn fail_open_falls_back_to_pty_on_unknown_bundle() {
    // Config loaded fine but the bundle is neither configured nor a built-in slug:
    // strict resolution errors, fail-open degrades to PTY.
    let cfg = crate::harness::config::HarnessesConfig::default();
    assert_eq!(
        resolve_transport_fail_open_with("not-a-harness", Ok(cfg)),
        crate::harness::Transport::Pty
    );
}

#[test]
fn fail_open_builtin_slug_stays_pty() {
    let cfg = crate::harness::config::HarnessesConfig::default();
    assert_eq!(
        resolve_transport_fail_open_with("claude", Ok(cfg)),
        crate::harness::Transport::Pty
    );
}

#[test]
fn pty_transport_reports_pty_kind() {
    assert_eq!(PtyTransport.kind(), TransportKind::Pty);
}

#[test]
fn acp_transport_reports_acp_kind() {
    assert_eq!(AcpTransport.kind(), TransportKind::Acp);
}

// ── defect #1: ACP resolves its harness from the BUNDLE, not the agent slug ────

/// An agent whose slug differs from its harness bundle name resolves the correct
/// harness/driver. Before the fix, `AcpTransport::spawn_child` passed `spec.slug`
/// (the AGENT slug) to `resolve_with`, so any agent with `slug != bundle` — e.g.
/// `reviewer` running bundle `codex-acp` — bailed at launch because `reviewer` is
/// not a `harnesses.json` key. The transport must resolve from `spec.bundle`.
#[test]
fn acp_resolves_harness_from_bundle_not_slug() {
    use crate::harness::{self, config::HarnessesConfig, Transport};
    use crate::session::Harness;
    let json = r#"{ "codex-acp": { "harness": "codex", "transport": "app-server" } }"#;
    let cfg: HarnessesConfig = serde_json::from_str(json).unwrap();
    let spec = LaunchSpec {
        slug: "reviewer".into(),
        bundle: Some("codex-acp".into()),
        root: "chan".into(),
        abs_path: "/tmp".into(),
        group: None,
        ephemeral: false,
        base_command: vec!["codex".into()],
        pubkey: "33".repeat(32),
    };
    // The transport resolves from the BUNDLE name, never the slug.
    assert_eq!(super::acp::bundle_name(&spec), "codex-acp");
    let scratch = std::env::temp_dir().join(format!("acp-bundle-{}", std::process::id()));
    let resolved = harness::resolve_with(&cfg, super::acp::bundle_name(&spec), &scratch)
        .expect("bundle resolves to its driver");
    assert_eq!(resolved.harness, Harness::Codex);
    assert_eq!(resolved.transport, Transport::AppServer);
    assert_eq!(
        resolved.base_argv.first().map(String::as_str),
        Some("codex")
    );
    // Regression pin: the PRE-FIX behavior — resolving from the agent slug — fails
    // loud, which is exactly why an agent with slug != bundle never launched.
    assert!(
        harness::resolve_with(&cfg, &spec.slug, &scratch).is_err(),
        "resolving from the agent slug must fail; the transport must use the bundle"
    );
}

#[tokio::test]
async fn acp_is_live_false_for_unknown_endpoint() {
    let ep = EndpointRef {
        kind: TransportKind::Acp,
        endpoint_id: "acp-nope".into(),
    };
    assert!(!AcpTransport.is_live(&ep));
    // kill of an unregistered endpoint is a no-op, not an error.
    assert!(AcpTransport.kill(&ep).await.is_ok());
}
