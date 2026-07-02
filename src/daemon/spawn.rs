use super::log_path;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Fork a detached `tenex-edge __daemon`: own session (`setsid` via
/// `process_group(0)`), stdio → daemon.log, survives the parent exiting.
///
/// The binary is `current_exe()` so an upgraded binary spawns its own daemon
/// (the basis of the version-skew re-exec). `$TENEX_EDGE_BIN` overrides it —
/// used by tests (whose `current_exe()` is the test harness) and as an escape
/// hatch.
pub(super) fn spawn_detached_daemon() -> Result<()> {
    let exe = match std::env::var_os("TENEX_EDGE_BIN") {
        Some(p) => PathBuf::from(p),
        None => std::env::current_exe().context("locating own executable")?,
    };
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .context("opening daemon.log")?;
    let log_err = log.try_clone()?;
    let mut command = std::process::Command::new(&exe);
    command
        .arg("__daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log))
        .stderr(std::process::Stdio::from(log_err));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
        .spawn()
        .with_context(|| format!("spawning detached daemon from {}", exe.display()))?;
    Ok(())
}
