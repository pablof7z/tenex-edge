//! A synchronous (blocking) thin client for CLI verbs that are NOT `async`.
//!
//! The async [`super::client::Client`] can't be `block_on`'d from inside the
//! tokio runtime the CLI already runs on (that panics). Sync verbs instead use
//! this plain `std::os::unix::net::UnixStream` client: spawn-if-absent via the
//! same `connect_or_spawn` mechanics is async, so for the blocking path we do a
//! minimal connect-or-spawn against the socket directly. Blocking I/O on a
//! one-shot CLI invocation is fine.

use super::protocol::{protocol_version, PleaseExit};
use super::socket_path;
use super::spawn::spawn_detached_daemon;
use crate::config;
use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

const HANDSHAKE_IO_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_RESPONSE_IO_TIMEOUT: Duration = Duration::from_secs(2);
const SLOW_RESPONSE_IO_TIMEOUT: Duration = Duration::from_secs(25);

/// Connect to the daemon (spawning if absent), do the handshake, send one
/// request, and return the `ok` payload. Mirrors the async client's behavior
/// including the version-skew exit+respawn, but synchronously.
pub fn call(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    for _ in 0..5 {
        match try_call(method, &params) {
            Ok(Outcome::Ok(v)) => return Ok(v),
            Ok(Outcome::Err(_code, msg)) => bail!("{msg}"),
            Ok(Outcome::SkewExit) => {
                std::thread::sleep(Duration::from_millis(200));
                spawn_if_absent()?;
            }
            Err(e) => {
                if e.to_string().contains("is newer than this binary") {
                    return Err(e);
                }
                spawn_if_absent()?;
            }
        }
    }
    bail!("could not complete daemon call {method}")
}

/// One-shot call that NEVER spawns the daemon. For high-frequency fail-open
/// surfaces (the statusline) that must render nothing when no daemon is running
/// rather than booting one just to draw a line.
pub fn call_no_spawn(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    match try_call(method, &params)? {
        Outcome::Ok(v) => Ok(v),
        Outcome::Err(_code, msg) => bail!("{msg}"),
        Outcome::SkewExit => bail!("daemon protocol skew; awaiting respawn"),
    }
}

enum Outcome {
    Ok(serde_json::Value),
    Err(String, String),
    SkewExit,
}

fn try_call(method: &str, params: &serde_json::Value) -> Result<Outcome> {
    let stream = UnixStream::connect(socket_path()).context("connecting to daemon socket")?;
    stream.set_read_timeout(Some(HANDSHAKE_IO_TIMEOUT))?;
    stream.set_write_timeout(Some(HANDSHAKE_IO_TIMEOUT))?;
    let mut w = stream.try_clone()?;
    let mut r = BufReader::new(stream);

    // hello → welcome.
    writeln!(
        w,
        "{}",
        serde_json::json!({"protocol": protocol_version(), "client_version": env!("CARGO_PKG_VERSION")})
    )?;
    let mut welcome_line = String::new();
    if r.read_line(&mut welcome_line)? == 0 {
        bail!("daemon closed before welcome");
    }
    let welcome: serde_json::Value = serde_json::from_str(welcome_line.trim())?;
    let dproto = welcome["protocol"].as_u64().unwrap_or(0) as u32;
    if dproto < protocol_version() {
        // Older daemon under a newer binary: ask it to exit, then respawn.
        writeln!(
            w,
            "{}",
            serde_json::to_string(&PleaseExit {
                protocol: protocol_version()
            })?
        )?;
        let _ = w.flush();
        return Ok(Outcome::SkewExit);
    }
    if dproto > protocol_version() {
        let mine = protocol_version();
        bail!(
            "daemon protocol {dproto} is newer than this binary's {mine} — restart your tenex-edge session (or reinstall)"
        );
    }

    // request → response.
    writeln!(
        w,
        "{}",
        serde_json::json!({"id": 1, "method": method, "params": params})
    )?;
    r.get_ref()
        .set_read_timeout(Some(response_timeout(method)))?;
    let mut resp_line = String::new();
    if r.read_line(&mut resp_line)? == 0 {
        bail!("daemon closed the connection");
    }
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim())?;
    if let Some(err) = resp.get("error") {
        let code = err["code"].as_str().unwrap_or("error").to_string();
        let msg = err["message"].as_str().unwrap_or("").to_string();
        return Ok(Outcome::Err(code, msg));
    }
    Ok(Outcome::Ok(
        resp.get("ok").cloned().unwrap_or(serde_json::Value::Null),
    ))
}

fn response_timeout(method: &str) -> Duration {
    match method {
        // `tmux_spawn` may spend up to 20s provisioning the launch channel before
        // returning. Keep handshake probes short, but do not make launch retry
        // while the daemon is still doing the requested work.
        "tmux_spawn" => SLOW_RESPONSE_IO_TIMEOUT,
        _ => DEFAULT_RESPONSE_IO_TIMEOUT,
    }
}

/// Synchronous spawn-if-absent: under the startup `flock`, reclaim a stale
/// socket and spawn a detached daemon, then poll-connect.
fn spawn_if_absent() -> Result<()> {
    config::ensure_dir(&config::edge_home())?;

    let mut noted_wait = false;
    let wait_deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < wait_deadline {
        if daemon_answers_ping() {
            return Ok(());
        }
        if let Some(lock) = super::client::StartupLock::try_acquire()? {
            eprintln!("[tenex-edge] starting daemon...");
            let sock = socket_path();
            if sock.exists() {
                let _ = std::fs::remove_file(&sock);
            }
            spawn_detached_daemon()?;
            drop(lock);
            break;
        }
        if !noted_wait {
            eprintln!("[tenex-edge] waiting for daemon to finish startup...");
            noted_wait = true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let mut noted_ready = false;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if daemon_answers_ping() {
            return Ok(());
        }
        if !noted_ready {
            eprintln!("[tenex-edge] waiting for daemon to answer RPCs...");
            noted_ready = true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    bail!("daemon socket exists but did not answer the handshake within 30s");
}

fn daemon_answers_ping() -> bool {
    matches!(try_call("ping", &serde_json::json!({})), Ok(Outcome::Ok(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_spawn_gets_slow_response_budget() {
        assert!(response_timeout("tmux_spawn") > Duration::from_secs(20));
        assert_eq!(response_timeout("ping"), DEFAULT_RESPONSE_IO_TIMEOUT);
    }
}
