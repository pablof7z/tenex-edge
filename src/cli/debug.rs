mod data;
mod loader;
mod render;
mod state;
mod util;

// Re-export public items for cli.rs to use
pub use data::HookTailOpts;

// Wrapper to match the original signature
pub(super) fn hook_tail(opts: HookTailOpts) -> anyhow::Result<()> {
    state::hook_tail(opts)
}

#[cfg(test)]
mod tests {
    use super::data::SessionPane;
    use super::util::*;
    use std::collections::BTreeMap;

    #[test]
    fn command_session_prefers_env_then_explicit_flag() {
        let v = serde_json::json!({
            "env": {"TENEX_EDGE_SESSION": "env-session"},
            "command": {"explicit_session": "flag-session"},
            "process": {"cwd": "/tmp"}
        });
        assert_eq!(command_session(&v).as_deref(), Some("env-session"));

        let v = serde_json::json!({
            "env": {},
            "command": {"explicit_session": "flag-session"},
            "process": {"cwd": "/tmp"}
        });
        assert_eq!(command_session(&v).as_deref(), Some("flag-session"));
    }

    #[test]
    fn hook_session_accepts_codex_field_variants() {
        let v = serde_json::json!({"conversation_id": "codex-session"});
        assert_eq!(hook_session(&v).as_deref(), Some("codex-session"));
    }

    #[test]
    fn command_session_can_infer_unique_live_agent_project() {
        let mut panes = BTreeMap::new();
        panes.insert(
            "session-a".to_string(),
            SessionPane {
                session: "session-a".to_string(),
                project: "proj".to_string(),
                agent: "coder".to_string(),
                ..SessionPane::default()
            },
        );
        assert_eq!(
            infer_command_session(&panes, "coder", "proj").as_deref(),
            Some("session-a")
        );

        panes.insert(
            "session-b".to_string(),
            SessionPane {
                session: "session-b".to_string(),
                project: "proj".to_string(),
                agent: "coder".to_string(),
                ..SessionPane::default()
            },
        );
        assert_eq!(infer_command_session(&panes, "coder", "proj"), None);
    }
}
