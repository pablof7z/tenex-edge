use super::*;

const SPAWN_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

// ── spawn-if-absent (race-safe via flock) ────────────────────────────────────

/// Ensure a daemon is listening. Under the startup lock: re-check the socket,
/// reclaim a stale one, then spawn a detached daemon and poll-connect.
pub(super) async fn spawn_daemon_if_absent() -> Result<()> {
    config::ensure_dir(&config::edge_home())?;
    // Acquire the exclusive startup lock (blocks until other spawners finish).
    let lock = StartupLock::acquire()?;

    // Someone may have bound the socket while we waited for the lock.
    if probe_connect().await {
        return Ok(());
    }
    // Stale socket: file present but nobody answering → reclaim under the lock.
    let sock = socket_path();
    if sock.exists() {
        let _ = std::fs::remove_file(&sock);
    }

    spawn_detached_daemon()?;
    // Lock is released when `lock` drops (after spawn returns); the daemon
    // re-acquires it on its own startup.
    drop(lock);

    // Poll-connect until the daemon binds.
    let deadline = Instant::now() + SPAWN_CONNECT_TIMEOUT;
    while Instant::now() < deadline {
        if probe_connect().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    bail!("daemon did not come up within {SPAWN_CONNECT_TIMEOUT:?}")
}

/// Cheap liveness probe: can we open the socket at all?
async fn probe_connect() -> bool {
    UnixStream::connect(socket_path()).await.is_ok()
}

/// Fork a detached `tenex-edge __daemon`: own session (`setsid` via
/// `process_group(0)`), stdio → daemon.log, survives the parent exiting.
///
/// The binary is `current_exe()` so an upgraded binary spawns its own daemon
/// (the basis of the version-skew re-exec). `$TENEX_EDGE_BIN` overrides it —
/// used by tests (whose `current_exe()` is the test harness) and as an escape
/// hatch.
fn spawn_detached_daemon() -> Result<()> {
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
    let mut command = std::process::Command::new(exe);
    command
        .arg("__daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log))
        .stderr(std::process::Stdio::from(log_err));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0); // detach from the caller's process group
    }
    command.spawn().context("spawning detached daemon")?;
    Ok(())
}

/// RAII wrapper over an exclusive `flock` on `daemon.lock`. The lock is released
/// when the `Flock` guard drops (i.e. when this `StartupLock` drops).
pub struct StartupLock {
    _flock: nix::fcntl::Flock<std::fs::File>,
}

fn open_lock_file() -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(lock_path())
        .context("opening daemon.lock")
}

impl StartupLock {
    /// Blocking exclusive acquire (used by spawning clients).
    pub fn acquire() -> Result<Self> {
        let file = open_lock_file()?;
        let flock = nix::fcntl::Flock::lock(file, nix::fcntl::FlockArg::LockExclusive)
            .map_err(|(_, e)| anyhow::anyhow!("flock daemon.lock: {e}"))?;
        Ok(StartupLock { _flock: flock })
    }

    /// Non-blocking exclusive acquire: `Ok(Some)` if we got it, `Ok(None)` if
    /// held by a live daemon. Used by the daemon to detect an existing peer.
    pub fn try_acquire() -> Result<Option<Self>> {
        let file = open_lock_file()?;
        match nix::fcntl::Flock::lock(file, nix::fcntl::FlockArg::LockExclusiveNonblock) {
            Ok(flock) => Ok(Some(StartupLock { _flock: flock })),
            // EWOULDBLOCK (== EAGAIN on these platforms): another daemon holds it.
            Err((_, nix::errno::Errno::EWOULDBLOCK)) => Ok(None),
            Err((_, e)) => Err(anyhow::anyhow!("flock(NB) daemon.lock: {e}")),
        }
    }
}
