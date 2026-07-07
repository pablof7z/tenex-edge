use crate::daemon::server::DaemonState;
use crate::session_host::launch::project_abs_path;
use crate::session_host::registry::{
    apply_agent_def_args, build_headless_command, headless_shape_for_bin, resolve_spawn_entry,
};
use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) struct ExecLaunch {
    pub(crate) id: String,
    pub(crate) child: Child,
    pub(crate) log_path: PathBuf,
    pub(crate) started_at: u64,
}

impl ExecLaunch {
    pub(crate) fn pid(&self) -> i32 {
        self.child.id() as i32
    }
}

pub(crate) fn agent_supports_headless_exec(slug: &str) -> bool {
    resolve_spawn_entry(slug)
        .ok()
        .and_then(|(base, _)| {
            base.first()
                .and_then(|bin| headless_shape_for_bin(bin.as_str()))
        })
        .is_some()
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn spawn_agent_exec(
    state: &Arc<DaemonState>,
    slug: &str,
    project: &str,
    prompt: &str,
    resume_id: Option<&str>,
    base_override: Option<Vec<String>>,
    group: Option<&str>,
    client_cwd: Option<&Path>,
    ordinal: Option<u32>,
) -> Result<ExecLaunch> {
    let (base_command, agent_def) = match base_override {
        Some(cmd) => (cmd, None),
        None => resolve_spawn_entry(slug)?,
    };
    let base_command = if resume_id.is_some() {
        base_command
    } else {
        apply_agent_def_args(base_command, slug, agent_def)
    };
    let bin = base_command.first().map(String::as_str).unwrap_or("");
    let shape = headless_shape_for_bin(bin)
        .with_context(|| format!("agent {slug:?} does not support headless exec via {bin:?}"))?;
    let command = build_headless_command(&base_command, shape, resume_id, prompt);
    let abs_path = project_abs_path(state, project, client_cwd)?;
    spawn_process(slug, project, group, ordinal, &abs_path, command)
}

fn spawn_process(
    slug: &str,
    project: &str,
    group: Option<&str>,
    ordinal: Option<u32>,
    cwd: &str,
    command: Vec<String>,
) -> Result<ExecLaunch> {
    if command.is_empty() {
        anyhow::bail!("headless exec command must not be empty");
    }
    let id = exec_id(slug);
    let started_at = crate::util::now_secs();
    let dir = exec_session_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let log_path = dir.join(format!("{id}.log"));
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("opening {}", log_path.display()))?;
    let log_err = log.try_clone()?;

    let mut child_cmd = std::process::Command::new(&command[0]);
    child_cmd
        .args(&command[1..])
        .current_dir(cwd)
        .env("TENEX_EDGE_SPAWNED", "1")
        .env("TENEX_EDGE_AGENT", slug)
        .env_remove("TENEX_EDGE_SESSION")
        .env_remove("TENEX_EDGE_PTY_SESSION")
        .env_remove("TENEX_EDGE_PTY_SOCKET")
        .env_remove("CLAUDE_CODE_CHILD_SESSION")
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));
    if let Some(channel) = group.filter(|g| !g.is_empty()) {
        child_cmd.env("TENEX_EDGE_CHANNEL", channel);
    }
    if let Some(ord) = ordinal {
        child_cmd.env("TENEX_EDGE_ORDINAL", ord.to_string());
    }
    unsafe {
        child_cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = child_cmd
        .spawn()
        .with_context(|| format!("spawning headless exec for {slug} in {project}"))?;
    Ok(ExecLaunch {
        id,
        child,
        log_path,
        started_at,
    })
}

fn exec_session_dir() -> PathBuf {
    crate::config::edge_home().join("exec-sessions")
}

fn exec_id(agent: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis();
    let safe_agent = agent
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("exec-{safe_agent}-{now}-{}", std::process::id())
}
