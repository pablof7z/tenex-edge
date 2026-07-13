use super::*;
use crate::session::Harness;

fn tmp_scratch() -> std::path::PathBuf {
    std::env::temp_dir().join("tenex-edge-harness-test-scratch")
}

#[test]
fn every_declared_cell_looks_up() {
    // Sanity: each row in the table is reachable by its own key.
    for d in driver::all() {
        let got = driver::lookup(d.harness, d.transport).expect("row must be findable");
        assert_eq!(got.harness, d.harness);
        assert_eq!(got.transport, d.transport);
    }
}

#[test]
fn invalid_cell_is_absent() {
    // Codex has no native ACP; Grok is PTY-only.
    assert!(driver::lookup(Harness::Codex, Transport::Acp).is_none());
    assert!(driver::lookup(Harness::Grok, Transport::AppServer).is_none());
    assert!(driver::lookup(Harness::Opencode, Transport::AppServer).is_none());
}

#[test]
fn claude_acp_uses_adapter_not_binary() {
    let d = driver::lookup(Harness::ClaudeCode, Transport::Acp).unwrap();
    assert_eq!(
        d.base_argv,
        &["npx", "--yes", "@agentclientprotocol/claude-agent-acp"]
    );
    assert!(d
        .base_env
        .contains(&driver::EnvDirective::Remove("CLAUDECODE")));
    assert_eq!(d.resume, driver::ResumeMechanism::AcpSessionLoad);
    assert_eq!(d.turn, driver::TurnModel::RpcTurn);
}

#[test]
fn codex_app_server_steer_and_config_flags() {
    let d = driver::lookup(Harness::Codex, Transport::AppServer).unwrap();
    assert_eq!(d.base_argv, &["codex", "app-server"]);
    assert_eq!(d.steer, driver::SteerPrimitive::AppServerSteer);
    assert_eq!(
        d.profile,
        driver::ProfileMechanism::CliConfigFlags { flag: "-c" }
    );
    assert_eq!(d.resume, driver::ResumeMechanism::AppServerThreadResume);
}

#[test]
fn config_parses_bundles_and_rejects_unknown_harness() {
    let json = r#"{
        "claude-acp": { "harness": "claude", "transport": "acp",
                        "profile": { "permissions": { "defaultMode": "acceptEdits" } } },
        "codex": { "harness": "codex", "transport": "app-server",
                   "profile": { "model": "gpt-5-codex", "sandbox_mode": "workspace-write" } }
    }"#;
    let cfg: config::HarnessesConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.get("claude-acp").unwrap().harness, Harness::ClaudeCode);
    assert_eq!(cfg.get("codex").unwrap().transport, Transport::AppServer);

    let bad = r#"{ "x": { "harness": "gpt5", "transport": "acp" } }"#;
    assert!(serde_json::from_str::<config::HarnessesConfig>(bad).is_err());
}

#[test]
fn missing_config_file_is_empty() {
    let cfg = config::HarnessesConfig::load_from(std::path::Path::new(
        "/nonexistent/tenex-edge/harnesses.json",
    ))
    .unwrap();
    assert!(cfg.bundles.is_empty());
}

#[test]
fn resolve_falls_back_to_builtin_pty_for_bare_slug() {
    let cfg = config::HarnessesConfig::default();
    let r = resolve_with(&cfg, "claude", &tmp_scratch()).unwrap();
    assert_eq!(r.harness, Harness::ClaudeCode);
    assert_eq!(r.transport, Transport::Pty);
    assert_eq!(r.base_argv, vec!["claude".to_string()]);
    assert!(r.profile.extra_argv.is_empty());
}

#[test]
fn resolve_unknown_bundle_fails_loud() {
    let cfg = config::HarnessesConfig::default();
    assert!(resolve_with(&cfg, "not-a-harness", &tmp_scratch()).is_err());
}

#[test]
fn codex_profile_becomes_config_flags() {
    let json = r#"{ "cx": { "harness": "codex", "transport": "app-server",
                            "profile": { "model": "gpt-5-codex", "sandbox_mode": "workspace-write" } } }"#;
    let cfg: config::HarnessesConfig = serde_json::from_str(json).unwrap();
    let r = resolve_with(&cfg, "cx", &tmp_scratch()).unwrap();
    // base_argv = ["codex","app-server"] + -c pairs (sorted by BTree? no, object
    // order is preserved by serde_json). Assert both pairs present.
    let joined = r.base_argv.join(" ");
    assert!(joined.starts_with("codex app-server"));
    assert!(joined.contains("-c model=gpt-5-codex"));
    assert!(joined.contains("-c sandbox_mode=workspace-write"));
    assert!(r.profile.files.is_empty());
}

#[test]
fn opencode_acp_profile_becomes_scratch_file_and_env() {
    let json = r#"{ "oc": { "harness": "opencode", "transport": "acp",
                            "profile": { "model": "anthropic/claude-sonnet-4-5" } } }"#;
    let cfg: config::HarnessesConfig = serde_json::from_str(json).unwrap();
    let scratch = tmp_scratch();
    let r = resolve_with(&cfg, "oc", &scratch).unwrap();
    assert_eq!(r.profile.files.len(), 1);
    let (path, contents) = &r.profile.files[0];
    assert!(path.ends_with("opencode.json"));
    assert!(contents.contains("claude-sonnet-4-5"));
    assert!(r
        .profile
        .extra_env
        .iter()
        .any(|(k, _)| k == "OPENCODE_CONFIG"));
}

#[test]
fn grok_profile_is_unsupported() {
    let json = r#"{ "g": { "harness": "grok", "transport": "pty",
                          "profile": { "model": "x" } } }"#;
    let cfg: config::HarnessesConfig = serde_json::from_str(json).unwrap();
    assert!(resolve_with(&cfg, "g", &tmp_scratch()).is_err());
}
