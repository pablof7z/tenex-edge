use super::*;

// ── invite (spawn/resume into an explicit channel) ───────────────────────────

/// `tenex-edge invite --channel <channel> (--agent <slug[@backend]> | --session <id>)`
/// spawns a fresh agent session or resumes a prior one into an existing channel.
pub(super) async fn invite_target(
    channel: String,
    agent: Option<String>,
    session: Option<String>,
) -> Result<()> {
    let selector = agent
        .as_ref()
        .map(|a| format!("--agent {a}"))
        .or_else(|| session.as_ref().map(|s| format!("--session {s}")))
        .unwrap_or_default();
    let v = daemon_call_async(
        "invite",
        crate::cli::rpc_params(serde_json::json!({
            "channel": channel,
            "target_agent": agent,
            "session": session,
        })),
    )
    .await?;
    if v["ambiguous"].is_array() {
        let name = v["reference"].as_str().unwrap_or("");
        eprintln!("'{name}' is ambiguous — re-run with an exact --channel:");
        if let Some(refs) = v["ambiguous"].as_array() {
            for r in refs.iter().filter_map(|r| r.as_str()) {
                eprintln!("  tenex-edge invite --channel {r} {selector}");
            }
        }
        std::process::exit(2);
    }
    let slug = v["agent"].as_str().unwrap_or("session");
    let pty = v["pty_id"].as_str().unwrap_or("");
    let online = v["online_agent"].as_str().unwrap_or(slug);
    if pty.is_empty() {
        println!("{} is now online", online.bold());
    } else {
        println!("{} is now online (pty {})", online.bold(), pty.dimmed());
    }
    Ok(())
}
