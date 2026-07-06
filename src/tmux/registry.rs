use crate::tmux::pane::tmux_available;
use anyhow::{Context, Result};

use crate::identity::LaunchCommand;

pub(super) struct SpawnDef {
    /// Harness slug (matches agent_slug / TENEX_EDGE_AGENT).
    pub(super) slug: &'static str,
    /// Window name shown in the tmux status bar.
    pub(super) window_name: &'static str,
    /// Command to run (first word of the exec, plus args).
    command: &'static [&'static str],
}

/// How a harness's launch command is transformed into a resume invocation.
/// The base command is the agent's configured launch command (e.g. `["claude",
/// "--dangerously-skip-permissions"]`), so the user's own flags are preserved.
#[derive(Clone, Copy)]
pub(super) enum ResumeShape {
    /// Resume is a flag that composes with the launch flags: append `<flag> <id>`
    /// to the base command. claude: `--resume`, opencode: `--session`.
    AppendFlag(&'static str),
    /// Resume is a subcommand that must follow the binary: insert `<sub> <id>`
    /// right after argv[0], keeping the remaining launch flags after it. The
    /// flags ride on the subcommand's own parser. codex: `resume`.
    Subcommand(&'static str),
}

static SPAWN_DEFS: &[SpawnDef] = &[
    SpawnDef {
        slug: "claude",
        window_name: "claude·tenex-edge",
        command: &["claude"],
    },
    SpawnDef {
        slug: "codex",
        window_name: "codex·tenex-edge",
        command: &["codex"],
    },
    SpawnDef {
        slug: "opencode",
        window_name: "opencode·tenex-edge",
        command: &["opencode"],
    },
    SpawnDef {
        slug: "grok",
        window_name: "grok·tenex-edge",
        command: &["grok"],
    },
];

pub(super) fn find_spawn_def(slug: &str) -> Option<&'static SpawnDef> {
    SPAWN_DEFS.iter().find(|d| d.slug == slug)
}

pub(crate) fn builtin_spawn_commands() -> Vec<LaunchCommand> {
    SPAWN_DEFS
        .iter()
        .filter_map(|d| {
            LaunchCommand::new(d.slug, d.command.iter().map(|s| s.to_string()).collect())
        })
        .collect()
}

fn builtin_spawn_command_for_slug(slug: &str) -> Option<Vec<String>> {
    find_spawn_def(slug).map(|d| d.command.iter().map(|s| s.to_string()).collect())
}

pub(super) fn resume_shape_for_bin(bin: &str) -> Option<ResumeShape> {
    let name = std::path::Path::new(bin)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(bin);
    match name {
        "claude" => Some(ResumeShape::AppendFlag("--resume")),
        "codex" => Some(ResumeShape::Subcommand("resume")),
        "opencode" => Some(ResumeShape::AppendFlag("--session")),
        "grok" => Some(ResumeShape::AppendFlag("--resume")),
        _ => None,
    }
}

pub(super) fn build_resume_command(
    base: &[String],
    shape: ResumeShape,
    resume_id: &str,
) -> Vec<String> {
    match shape {
        ResumeShape::AppendFlag(flag) => {
            let mut out = base.to_vec();
            out.push(flag.to_string());
            out.push(resume_id.to_string());
            out
        }
        ResumeShape::Subcommand(sub) => {
            let mut out = Vec::with_capacity(base.len() + 2);
            let mut it = base.iter();
            if let Some(bin) = it.next() {
                out.push(bin.clone());
            }
            out.push(sub.to_string());
            out.push(resume_id.to_string());
            out.extend(it.cloned());
            out
        }
    }
}

/// Returns `(slug, display_command, byline)` tuples for agents tenex-edge has
/// an identity for. Returns an empty vec when tmux is absent.
pub fn spawnable_agents() -> Vec<(String, String, Option<String>)> {
    if !tmux_available() {
        tracing::debug!("spawnable_agents: tmux not available");
        return Vec::new();
    }
    let edge_home = crate::config::edge_home();
    let agents = crate::identity::list_local_agents(&edge_home);
    tracing::debug!(count = agents.len(), "spawnable_agents: agents in store");
    let result: Vec<(String, String, Option<String>)> = agents
        .into_iter()
        .filter_map(|(slug, commands, _agent_def, byline)| {
            let display_cmd = commands
                .first()
                .map(|c| c.display())
                .or_else(|| find_spawn_def(&slug).map(|d| d.command.join(" ")));
            tracing::debug!(slug = %slug, display_cmd = ?display_cmd, "spawnable_agents: candidate");
            Some((slug, display_cmd?, byline))
        })
        .collect();
    tracing::debug!(?result, "spawnable_agents: result");
    result
}

/// Resolve the base harness command and inline agent definition for `slug`.
/// The first configured `commands` entry takes priority, with SPAWN_DEFS as
/// fallback. The removed singular `command` field is intentionally ignored.
pub(super) fn resolve_spawn_entry(slug: &str) -> Result<(Vec<String>, Option<serde_json::Value>)> {
    let edge_home = crate::config::edge_home();
    let entry = crate::identity::list_local_agents(&edge_home)
        .into_iter()
        .find(|(s, _, _, _)| s == slug);
    let (file_cmd, agent_def) = entry
        .map(|(_, commands, def, _)| (commands.first().map(|c| c.argv.clone()), def))
        .unwrap_or((None, None));
    let base = file_cmd
        .or_else(|| builtin_spawn_command_for_slug(slug))
        .with_context(|| format!("no harness command for agent {slug:?}: add a \"commands\" field to ~/.tenex-edge/agents/{slug}.json"))?;
    Ok((base, agent_def))
}

/// Append harness-specific args for the inline agent definition.
pub(super) fn apply_agent_def_args(
    mut cmd: Vec<String>,
    slug: &str,
    agent_def: Option<serde_json::Value>,
) -> Vec<String> {
    let Some(def) = agent_def else { return cmd };
    let bin = cmd.first().map(String::as_str).unwrap_or("");
    if bin == "claude" {
        let mut wrapper = serde_json::Map::new();
        wrapper.insert(slug.to_string(), def);
        if let Ok(json) = serde_json::to_string(&serde_json::Value::Object(wrapper)) {
            cmd.push("--agents".to_string());
            cmd.push(json);
            cmd.push("--agent".to_string());
            cmd.push(slug.to_string());
        }
    }
    cmd
}
