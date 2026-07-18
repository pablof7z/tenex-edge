use super::meta::{session_socket, LaunchMetadata};
use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SpawnSessionArgs {
    pub id: Option<String>,
    pub agent: String,
    pub root: String,
    pub cwd: PathBuf,
    pub channel: Option<String>,
    pub session_name: Option<String>,
    pub ephemeral: bool,
    pub command: Vec<String>,
    pub env: Vec<(String, String)>,
    pub env_remove: Vec<String>,
}

pub fn spawn_session(args: SpawnSessionArgs) -> Result<LaunchMetadata> {
    if args.command.is_empty() {
        anyhow::bail!("pty launch command must not be empty");
    }
    let id = args.id.unwrap_or_else(|| new_endpoint_id(&args.agent));
    let instance_token = unique_token();
    let socket = session_socket(&id);
    let log_path = super::meta::session_dir().join(format!("{id}.supervisor.log"));
    std::fs::create_dir_all(super::meta::session_dir())
        .context("creating pty session directory")?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("opening {}", log_path.display()))?;
    let log_err = log.try_clone()?;
    let bin = std::env::current_exe().context("locating current mosaico executable")?;
    let mut child = std::process::Command::new(bin);
    child
        .arg("__pty-supervisor")
        .arg("--id")
        .arg(&id)
        .arg("--instance-token")
        .arg(&instance_token)
        .arg("--socket")
        .arg(&socket)
        .arg("--cwd")
        .arg(&args.cwd)
        .arg("--agent")
        .arg(&args.agent);
    if let Some(channel) = &args.channel {
        child.arg("--channel").arg(channel);
    }
    if let Some(session_name) = &args.session_name {
        child.arg("--session-name").arg(session_name);
    }
    if args.ephemeral {
        child.arg("--ephemeral");
    }
    for (key, value) in &args.env {
        child.env(key, value);
    }
    for key in &args.env_remove {
        child.env_remove(key);
    }
    child.arg("--").args(&args.command);
    child
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));
    unsafe {
        child.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let supervisor = child.spawn().context("spawning portable-pty supervisor")?;

    let meta = LaunchMetadata {
        id,
        socket: socket.to_string_lossy().to_string(),
        supervisor_pid: supervisor.id(),
        instance_token,
        adopted_process_fingerprint: String::new(),
        child_pid: None,
        agent: args.agent,
        root: args.root,
        cwd: args.cwd.to_string_lossy().to_string(),
        ephemeral: args.ephemeral,
        command: args.command,
    };
    super::meta::write_metadata(&meta)?;
    Ok(meta)
}

pub(crate) fn new_endpoint_id(agent: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let safe_agent = agent
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(16)
        .collect::<String>();
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{safe_agent}-{now}-{}-{}-{sequence}",
        std::process::id(),
        process_nonce()
    )
}

fn process_nonce() -> &'static str {
    static NONCE: OnceLock<String> = OnceLock::new();
    NONCE
        .get_or_init(|| unique_token().chars().take(16).collect())
        .as_str()
}

fn unique_token() -> String {
    use std::io::Read;

    let mut bytes = [0_u8; 16];
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut random| random.read_exact(&mut bytes))
        .is_ok()
    {
        return bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    static FALLBACK_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "fallback-{nanos:x}-{}-{}",
        std::process::id(),
        FALLBACK_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn endpoint_ids_are_unique_within_the_same_process_and_second() {
        let first = super::new_endpoint_id("grok");
        let second = super::new_endpoint_id("grok");
        assert_ne!(first, second);
    }

    #[cfg(unix)]
    #[test]
    fn generated_endpoint_stays_within_unix_socket_limits() {
        use std::os::unix::ffi::OsStrExt;

        let id = super::new_endpoint_id(&"very-long-agent-name-".repeat(8));
        assert!(super::session_socket(&id).as_os_str().as_bytes().len() < 100);
    }
}
