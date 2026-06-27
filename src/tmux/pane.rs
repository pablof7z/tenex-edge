use crate::daemon::server::DaemonState;
use anyhow::{Context, Result};
use std::sync::Arc;

pub(super) fn tmux_available() -> bool {
    std::process::Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Verify that `pane_id` (e.g. "%5") is still alive.
/// Returns the current command running in the pane on success (e.g. "claude").
pub fn pane_alive_pub(pane_id: &str) -> Option<String> {
    pane_alive(pane_id)
}

pub(super) fn pane_alive(pane_id: &str) -> Option<String> {
    let out = std::process::Command::new("tmux")
        .args([
            "display",
            "-p",
            "-t",
            pane_id,
            "#{pane_id} #{pane_current_command}",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let cmd = s
        .split_once(' ')
        .map(|(_, rest)| rest)
        .unwrap_or("")
        .to_string();
    Some(cmd)
}

fn session_of_pane(pane_id: &str, socket: Option<&str>) -> Option<String> {
    let mut cmd = std::process::Command::new("tmux");
    if let Some(s) = socket.filter(|s| !s.is_empty()) {
        cmd.args(["-S", s]);
    }
    let out = cmd
        .args(["display-message", "-p", "-t", pane_id, "#{session_name}"])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Set the `@te_session` tmux user option on the session owning `pane_id` to
/// `session_id`, so status-format can resolve the pane's daemon session.
pub fn set_pane_session_id(pane_id: &str, session_id: &str, socket: Option<&str>) {
    let Some(session) = session_of_pane(pane_id, socket) else {
        if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
            eprintln!("[tmux] set_pane_session_id: pane {pane_id} not found in any tmux session");
        }
        return;
    };
    let mut cmd = std::process::Command::new("tmux");
    if let Some(s) = socket.filter(|s| !s.is_empty()) {
        cmd.args(["-S", s]);
    }
    let status = cmd
        .args(["set-option", "-t", &session, "@te_session", session_id])
        .status();
    match status {
        Ok(s) if s.success() => {}
        Err(e) => {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[tmux] set-option @te_session failed: {e}");
            }
        }
        Ok(s) => {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[tmux] set-option @te_session exited {s}");
            }
        }
    }
}

pub(super) async fn send_enter(pane_id: &str) -> Result<()> {
    let status = tokio::process::Command::new("tmux")
        .args(["send-keys", "-t", pane_id, "Enter"])
        .status()
        .await
        .context("tmux send-keys Enter")?;
    if !status.success() {
        anyhow::bail!("tmux send-keys Enter failed for pane {pane_id}");
    }
    Ok(())
}

pub(super) async fn paste_text(pane_id: &str, text: &str) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    const BUF: &str = "te-spawn-msg";

    let mut child = tokio::process::Command::new("tmux")
        .args(["load-buffer", "-b", BUF, "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("tmux load-buffer spawn")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .await
            .context("writing tmux paste buffer")?;
    }
    let status = child.wait().await.context("tmux load-buffer wait")?;
    if !status.success() {
        anyhow::bail!("tmux load-buffer failed for pane {pane_id}");
    }

    let status = tokio::process::Command::new("tmux")
        .args(["paste-buffer", "-p", "-d", "-b", BUF, "-t", pane_id])
        .status()
        .await
        .context("tmux paste-buffer")?;
    if !status.success() {
        anyhow::bail!("tmux paste-buffer failed for pane {pane_id}");
    }
    Ok(())
}

pub struct EndpointStatus {
    pub session_id: String,
    pub pane_id: String,
    pub pane_command: String,
    pub alive: bool,
    pub registered_at: u64,
    pub last_verified: u64,
}

/// List all registered tmux endpoints with liveness.
pub fn list_endpoint_statuses(state: &Arc<DaemonState>) -> Vec<EndpointStatus> {
    let endpoints =
        state.with_store(|s| s.list_session_endpoints_of_kind("tmux").unwrap_or_default());

    endpoints
        .into_iter()
        .map(|ep| {
            let cmd_opt = pane_alive(&ep.target);
            EndpointStatus {
                session_id: ep.session_id,
                pane_id: ep.target,
                pane_command: cmd_opt.clone().unwrap_or_default(),
                alive: cmd_opt.is_some(),
                registered_at: ep.registered_at,
                last_verified: ep.last_verified,
            }
        })
        .collect()
}
