use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchMetadata {
    pub id: String,
    pub socket: String,
    pub supervisor_pid: u32,
    #[serde(default)]
    pub instance_token: String,
    pub agent: String,
    pub root: String,
    pub cwd: String,
    #[serde(default)]
    pub ephemeral: bool,
    pub command: Vec<String>,
}

pub fn session_dir() -> PathBuf {
    crate::config::mosaico_home().join("pty")
}

pub fn session_socket(id: &str) -> PathBuf {
    socket_dir_for(&crate::config::mosaico_home(), current_uid()).join(format!("{id}.sock"))
}

fn socket_dir_for(mosaico_home: &std::path::Path, uid: u32) -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/tmp")
            .join(format!("mosaico-pty-{uid}"))
            .join(mosaico_home_hash(mosaico_home))
    }
    #[cfg(not(unix))]
    {
        std::env::temp_dir()
            .join(format!("mosaico-pty-{uid}"))
            .join(mosaico_home_hash(mosaico_home))
    }
}

#[cfg(unix)]
fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

#[cfg(not(unix))]
fn current_uid() -> u32 {
    0
}

fn mosaico_home_hash(mosaico_home: &std::path::Path) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in mosaico_home.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn metadata_path(id: &str) -> PathBuf {
    session_dir().join(format!("{id}.json"))
}

pub fn write_metadata(meta: &LaunchMetadata) -> Result<()> {
    std::fs::create_dir_all(session_dir()).context("creating pty session directory")?;
    let bytes = serde_json::to_vec_pretty(meta)?;
    std::fs::write(metadata_path(&meta.id), bytes).context("writing pty metadata")
}

pub fn remove_metadata(id: &str) -> Result<()> {
    match std::fs::remove_file(metadata_path(id)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).context("removing pty metadata"),
    }
}

pub fn read_all_metadata() -> Vec<LaunchMetadata> {
    let Ok(entries) = std::fs::read_dir(session_dir()) else {
        return Vec::new();
    };
    let mut out = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .filter_map(|e| std::fs::read(e.path()).ok())
        .filter_map(|bytes| serde_json::from_slice::<LaunchMetadata>(&bytes).ok())
        .collect::<Vec<_>>();
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out
}

/// Terminate only the supervisor whose persisted metadata and live command line
/// both identify `endpoint_id`. The identity check prevents a recycled PID from
/// turning stale metadata into an unrelated process kill.
pub(crate) fn terminate_owned_supervisor(endpoint_id: &str) -> Result<bool> {
    let Some(metadata) = read_all_metadata()
        .into_iter()
        .find(|metadata| metadata.id == endpoint_id)
    else {
        return Ok(false);
    };
    let pid = i32::try_from(metadata.supervisor_pid).context("supervisor pid overflow")?;
    if pid <= 1 || !process_exists(pid) {
        remove_metadata(endpoint_id)?;
        return Ok(true);
    }
    verify_owned_process(pid, endpoint_id, &metadata.instance_token)?;
    signal(pid, nix::sys::signal::Signal::SIGTERM)?;
    wait_for_exit(pid, 20);
    if process_exists(pid) {
        verify_owned_process(pid, endpoint_id, &metadata.instance_token)?;
        signal(pid, nix::sys::signal::Signal::SIGKILL)?;
        wait_for_exit(pid, 20);
    }
    if process_exists(pid) {
        anyhow::bail!("PTY supervisor {endpoint_id:?} pid {pid} did not terminate");
    }
    remove_metadata(endpoint_id)?;
    Ok(true)
}

fn signal(pid: i32, signal: nix::sys::signal::Signal) -> Result<()> {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), Some(signal))
        .with_context(|| format!("sending {signal:?} to PTY supervisor pid {pid}"))
}

fn wait_for_exit(pid: i32, attempts: usize) {
    for _ in 0..attempts {
        if !process_exists(pid) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn verify_owned_process(pid: i32, endpoint_id: &str, instance_token: &str) -> Result<()> {
    let output = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .context("inspecting PTY supervisor command")?;
    let command = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || !command_owns_endpoint(&command, endpoint_id, instance_token) {
        anyhow::bail!("refusing to terminate pid {pid}: command does not own PTY {endpoint_id:?}");
    }
    Ok(())
}

fn command_owns_endpoint(command: &str, endpoint_id: &str, instance_token: &str) -> bool {
    if endpoint_id.is_empty() || instance_token.is_empty() {
        return false;
    }
    let Some(argv) = shlex::split(command.trim()) else {
        return false;
    };
    if argv.get(1).map(String::as_str) != Some("__pty-supervisor") {
        return false;
    }
    let supervisor_args = &argv[2..];
    let option_end = supervisor_args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(supervisor_args.len());
    let supervisor_options = &supervisor_args[..option_end];
    exact_option(supervisor_options, "--id") == Some(endpoint_id)
        && exact_option(supervisor_options, "--instance-token") == Some(instance_token)
}

fn exact_option<'a>(argv: &'a [String], option: &str) -> Option<&'a str> {
    argv.windows(2)
        .find(|pair| pair[0] == option)
        .map(|pair| pair[1].as_str())
}

fn process_exists(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

/// Resolve a PTY endpoint id to its supervisor socket without inventing a
/// socket for non-PTY endpoints. Absolute paths are accepted for direct callers.
pub fn endpoint_socket(endpoint_id: &str) -> Option<String> {
    endpoint_socket_in(endpoint_id, read_all_metadata())
}

fn endpoint_socket_in(
    endpoint_id: &str,
    metadata: impl IntoIterator<Item = LaunchMetadata>,
) -> Option<String> {
    if std::path::Path::new(endpoint_id).is_absolute() {
        return Some(endpoint_id.to_string());
    }
    metadata
        .into_iter()
        .find(|meta| meta.id == endpoint_id)
        .map(|meta| meta.socket)
}

pub fn resolve_socket(id_or_path: &str) -> PathBuf {
    let path = PathBuf::from(id_or_path);
    if path.components().count() > 1 || id_or_path.ends_with(".sock") {
        path
    } else if let Ok(bytes) = std::fs::read(metadata_path(id_or_path)) {
        serde_json::from_slice::<LaunchMetadata>(&bytes)
            .ok()
            .map(|meta| PathBuf::from(meta.socket))
            .unwrap_or_else(|| session_socket(id_or_path))
    } else {
        session_socket(id_or_path)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn endpoint_socket_comes_from_launch_metadata() {
        let meta = super::LaunchMetadata {
            id: "pty-1".into(),
            socket: "/tmp/pty-1.sock".into(),
            supervisor_pid: 42,
            instance_token: "token-1".into(),
            agent: "agent".into(),
            root: "/tmp".into(),
            cwd: "/tmp".into(),
            ephemeral: false,
            command: vec!["codex".into()],
        };

        assert_eq!(
            super::endpoint_socket_in("pty-1", [meta]),
            Some("/tmp/pty-1.sock".into())
        );
        assert_eq!(super::endpoint_socket_in("acp-1", std::iter::empty()), None);
    }

    #[cfg(unix)]
    #[test]
    fn socket_path_stays_short_for_long_mosaico_home() {
        use std::os::unix::ffi::OsStrExt;

        let mosaico_home = std::path::Path::new(
            "/var/folders/kx/13lj0yd976x0tn90z1ntqbn80000gn/T/mosaico-e2e/mosaico-b/mosaico",
        );
        let path =
            super::socket_dir_for(mosaico_home, 501).join("testing-lead-1783399436-28334.sock");

        assert!(path.as_os_str().as_bytes().len() < 100);
    }

    #[test]
    fn ownership_requires_exact_endpoint_and_instance_token_arguments() {
        let command =
            "/opt/mosaico __pty-supervisor --id grok-123-456 --instance-token token-2 -- echo";
        assert!(super::command_owns_endpoint(
            command,
            "grok-123-456",
            "token-2"
        ));
        assert!(!super::command_owns_endpoint(
            command,
            "grok-123-45",
            "token-2"
        ));
        assert!(!super::command_owns_endpoint(
            command,
            "grok-123-456",
            "token"
        ));
        assert!(!super::command_owns_endpoint(
            "/opt/mosaico unrelated -- __pty-supervisor --id grok-123-456 --instance-token token-2",
            "grok-123-456",
            "token-2"
        ));
        assert!(!super::command_owns_endpoint(
            "/opt/mosaico __pty-supervisor --id other --instance-token other -- --id grok-123-456 --instance-token token-2",
            "grok-123-456",
            "token-2"
        ));
    }

    #[test]
    fn old_metadata_without_instance_token_remains_readable_but_untrusted() {
        let metadata: super::LaunchMetadata = serde_json::from_value(serde_json::json!({
            "id": "grok-old",
            "socket": "/tmp/grok-old.sock",
            "supervisor_pid": 42,
            "agent": "grok",
            "root": "/tmp",
            "cwd": "/tmp",
            "command": ["grok"]
        }))
        .expect("old metadata should remain readable for live-session adoption");

        assert!(metadata.instance_token.is_empty());
        assert!(!super::command_owns_endpoint(
            "/opt/mosaico __pty-supervisor --id grok-old -- grok",
            &metadata.id,
            &metadata.instance_token,
        ));
    }
}
