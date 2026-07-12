use super::*;

#[derive(serde::Deserialize)]
struct PtySpawnParams {
    agent: String,
    root: String,
    #[serde(default)]
    command: Vec<String>,
    /// Override the entire base command, replacing what would be resolved from
    /// the agent file. When non-empty, `command` (extra args) is still appended.
    #[serde(default)]
    base_command: Vec<String>,
    /// The client's cwd, forwarded so the daemon spawns the agent in the
    /// directory the user actually invoked `tenex-edge launch` from.
    #[serde(default)]
    cwd: Option<String>,
    /// The resolved opaque channel id to scope the spawned session into.
    #[serde(default)]
    channel: Option<String>,
    /// Operator-selected public handle prefix from `launch --name`.
    #[serde(default)]
    session_name: Option<String>,
    /// Optional initial prompt to open the fresh session on. Used by the headless
    /// launch path, where the child lives in the daemon.
    #[serde(default)]
    prompt: Option<String>,
}

pub(in crate::daemon::server) async fn rpc_pty_spawn(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: PtySpawnParams =
        serde_json::from_value(params.clone()).context("parsing pty_spawn params")?;
    let client_cwd = p.cwd.as_deref().map(std::path::Path::new);
    let base_override = (!p.base_command.is_empty()).then_some(p.base_command);
    let group = p.channel.as_deref();
    if let Some(name) = p.session_name.as_deref().filter(|name| !name.is_empty()) {
        state.with_store(|s| s.ensure_custom_handle_available(&p.agent, name))?;
    }
    super::provision_before_spawn(state, &p.agent, &p.root, group).await?;
    let pty_id = crate::session_host::spawn_agent(
        state,
        &p.agent,
        &p.root,
        p.command,
        base_override,
        group,
        client_cwd,
        p.session_name.as_deref(),
    )
    .await?;

    if let Some(prompt) = p.prompt.as_deref().filter(|prompt| !prompt.is_empty()) {
        crate::session_host::deliver_spawn_prompt(&p.agent, &pty_id, prompt).await;
    }
    Ok(serde_json::json!({ "pty_id": pty_id, "agent": p.agent, "root": p.root }))
}
