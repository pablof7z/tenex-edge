use super::*;

// ── who ──────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
struct WhoParams {
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    all: bool,
    #[serde(default)]
    all_projects: bool,
    #[serde(default)]
    cwd: Option<String>,
}

/// `who`: build the snapshot with the SAME function the CLI used. The client
/// renders it with the existing renderers, so output is byte-identical. The
/// daemon resolves the current project the same way the old CLI did.
pub(super) fn rpc_who(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: WhoParams = serde_json::from_value(params.clone()).unwrap_or_default();
    let current_project = if p.all_projects {
        None
    } else {
        Some(p.project.clone().unwrap_or_else(|| {
            let cwd = p
                .cwd
                .clone()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            crate::project::resolve(&cwd)
        }))
    };
    let now = now_secs();
    let host = state.host.clone();
    let snapshot = state.with_store(|s| {
        crate::cli::load_who_snapshot(s, current_project.as_deref(), p.all, now, &host)
    })?;
    Ok(serde_json::to_value(snapshot)?)
}
