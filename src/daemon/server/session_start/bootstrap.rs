use super::*;

pub(crate) struct PtySessionStart<'a> {
    pub(crate) pubkey: &'a str,
    pub(crate) reclaimed_pubkey: Option<&'a str>,
    pub(crate) channel: Option<&'a str>,
    pub(crate) channels: &'a [String],
    pub(crate) resume_id: Option<&'a str>,
    pub(crate) dispatch_event: Option<&'a str>,
    pub(crate) session_name: Option<&'a str>,
}

pub(crate) async fn bootstrap_pty_session_start(
    state: &Arc<DaemonState>,
    meta: &crate::pty::LaunchMetadata,
    request: PtySessionStart<'_>,
) -> Result<String> {
    let harness = infer_harness(&meta.command);
    let watch_pid = i32::try_from(meta.supervisor_pid).ok();
    let response = rpc_session_start(
        state,
        &serde_json::json!({
            "agent": &meta.agent,
            "pubkey": request.pubkey,
            "reclaimed_pubkey": request.reclaimed_pubkey,
            "harness": harness.as_str(),
            "cwd": &meta.cwd,
            "channel": request.channel,
            "channels": request.channels,
            "watch_pid": watch_pid,
            "pty_session": &meta.id,
            "endpoint_kind": if meta.socket.is_empty() { "acp" } else { "pty" },
            "resume_id": request.resume_id,
            "dispatch_event": request.dispatch_event,
            "session_name": request.session_name,
        }),
        None,
    )
    .await?;
    private_run_for_public_response(state, &response)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn bootstrap_exec_session_start(
    state: &Arc<DaemonState>,
    agent: &str,
    harness: Harness,
    cwd: &str,
    channel: Option<&str>,
    watch_pid: i32,
    native_id: Option<&str>,
    pubkey: &str,
) -> Result<String> {
    let response = rpc_session_start(
        state,
        &serde_json::json!({
            "agent": agent,
            "pubkey": pubkey,
            "harness": harness.as_str(),
            "cwd": cwd,
            "channel": channel,
            "watch_pid": watch_pid,
            "harness_session": native_id,
        }),
        None,
    )
    .await?;
    private_run_for_public_response(state, &response)
}

fn private_run_for_public_response(
    state: &Arc<DaemonState>,
    response: &serde_json::Value,
) -> Result<String> {
    let pubkey = response["pubkey"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("session_start bootstrap returned no pubkey"))?;
    state
        .with_store(|store| store.get_session(pubkey))?
        .map(|session| session.pubkey)
        .ok_or_else(|| anyhow::anyhow!("session_start created no runtime for pubkey {pubkey}"))
}

pub(crate) fn infer_harness(command: &[String]) -> Harness {
    // The ACP path records the real argv (defect #8). The claude-agent-acp adapter
    // launches through `npx --yes @agentclientprotocol/claude-agent-acp`, so
    // argv[0] is `npx`, not
    // `claude` — recognize the adapter package to keep the harness correct.
    if command.iter().any(|a| a.contains("claude-agent-acp")) {
        return Harness::ClaudeCode;
    }
    match command_binary(command).as_deref() {
        Some("claude" | "claude-code") => Harness::ClaudeCode,
        Some("codex") => Harness::Codex,
        Some("opencode") => Harness::Opencode,
        Some("grok") => Harness::Grok,
        _ => Harness::Unknown,
    }
}

fn command_binary(command: &[String]) -> Option<String> {
    let mut index = 0;
    if command.first().is_some_and(|arg| base_name(arg) == "env") {
        index = 1;
        while let Some(arg) = command.get(index) {
            if arg == "-u" || arg == "--unset" {
                index += 2;
                continue;
            }
            if arg.starts_with("-u") && arg.len() > 2 {
                index += 1;
                continue;
            }
            if arg == "-i" || arg == "--ignore-environment" {
                index += 1;
                continue;
            }
            if arg.contains('=') {
                index += 1;
                continue;
            }
            break;
        }
    }
    command.get(index).map(|arg| base_name(arg))
}

fn base_name(arg: &str) -> String {
    std::path::Path::new(arg)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(arg)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_wrapped_codex_command() {
        let command = vec![
            "env".to_string(),
            "-u".to_string(),
            "CLAUDE_CODE_SESSION_ID".to_string(),
            "TENEX_EDGE_ORDINAL=1".to_string(),
            "/usr/local/bin/codex".to_string(),
            "--yolo".to_string(),
        ];

        assert_eq!(infer_harness(&command), Harness::Codex);
    }
}
