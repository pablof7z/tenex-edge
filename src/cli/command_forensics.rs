use anyhow::Result;
use clap::error::Error as ClapError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const LOG_SCHEMA: &str = "tenex-edge.command-call.v1";
pub const COMMAND_CALL_LOG_ENV: &str = "TENEX_EDGE_COMMAND_CALL_LOG";

pub struct CommandCallLog {
    path: Option<PathBuf>,
    call_id: Option<String>,
}

impl CommandCallLog {
    pub fn start(argv: &[String]) -> Self {
        let Some(path) = configured_log_path() else {
            return Self {
                path: None,
                call_id: None,
            };
        };
        let call_id = call_id();
        let payload = serde_json::json!({
            "schema": LOG_SCHEMA,
            "phase": "received",
            "call_id": call_id,
            "timestamp": timestamp(),
            "command": {
                "argv": argv,
                "line": argv.join(" "),
                "subcommand": argv.get(1).cloned().unwrap_or_default(),
                "explicit_session": flag_value(argv, "--session"),
            },
            "process": process_snapshot(),
            "parent_chain": parent_chain(),
            "env": redacted_env(),
        });
        append_json(Some(&path), &payload);
        Self {
            path: Some(path),
            call_id: Some(call_id),
        }
    }

    pub fn finish_clap_error(&self, err: &ClapError) {
        let Some(call_id) = self.call_id.as_ref() else {
            return;
        };
        let payload = serde_json::json!({
            "schema": LOG_SCHEMA,
            "phase": "finished",
            "call_id": call_id,
            "timestamp": timestamp(),
            "result": {
                "ok": err.exit_code() == 0,
                "exit_code": err.exit_code(),
                "kind": format!("{:?}", err.kind()),
                "error": err.to_string(),
            },
        });
        append_json(self.path.as_ref(), &payload);
    }

    pub fn finish_runtime_error(&self, message: &str) {
        let Some(call_id) = self.call_id.as_ref() else {
            return;
        };
        let payload = serde_json::json!({
            "schema": LOG_SCHEMA,
            "phase": "finished",
            "call_id": call_id,
            "timestamp": timestamp(),
            "result": {
                "ok": false,
                "exit_code": 1,
                "error": message,
            },
        });
        append_json(self.path.as_ref(), &payload);
    }

    pub fn finish_result(&self, result: &Result<()>) {
        let Some(call_id) = self.call_id.as_ref() else {
            return;
        };
        let payload = serde_json::json!({
            "schema": LOG_SCHEMA,
            "phase": "finished",
            "call_id": call_id,
            "timestamp": timestamp(),
            "result": match result {
                Ok(()) => serde_json::json!({ "ok": true, "exit_code": 0 }),
                Err(e) => serde_json::json!({
                    "ok": false,
                    "exit_code": 1,
                    "error": format!("{e:#}"),
                }),
            },
        });
        append_json(self.path.as_ref(), &payload);
    }
}

pub(crate) fn configured_log_path() -> Option<PathBuf> {
    configured_log_path_from(std::env::var(COMMAND_CALL_LOG_ENV).ok().as_deref())
}

fn configured_log_path_from(raw: Option<&str>) -> Option<PathBuf> {
    let trimmed = raw?.trim();
    if matches!(trimmed, "" | "0" | "false" | "off" | "none") {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn append_json(path: Option<&PathBuf>, payload: &Value) {
    let Some(path) = path else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = serde_json::to_writer(&mut file, payload);
        let _ = writeln!(file);
    }
}

fn flag_value(argv: &[String], flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    let mut it = argv.iter().peekable();
    while let Some(arg) = it.next() {
        if let Some(value) = arg.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
        if arg == flag {
            return it.peek().map(|v| (*v).to_string());
        }
    }
    None
}

fn call_id() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}.{:09}-{}",
        d.as_secs(),
        d.subsec_nanos(),
        std::process::id()
    )
}

fn timestamp() -> Value {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    serde_json::json!({
        "unix_ms": d.as_millis(),
        "utc": format_utc(d.as_secs()),
    })
}

fn format_utc(unix_secs: u64) -> String {
    unsafe {
        let t = unix_secs as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::gmtime_r(&t, &mut tm).is_null() {
            return unix_secs.to_string();
        }
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec,
        )
    }
}

fn process_snapshot() -> Value {
    serde_json::json!({
        "pid": std::process::id(),
        "ppid": ps_ppid(std::process::id() as i32),
        "exe": std::env::current_exe().ok().map(|p| p.display().to_string()),
        "cwd": std::env::current_dir().ok().map(|p| p.display().to_string()),
    })
}

fn parent_chain() -> Vec<Value> {
    let mut out = Vec::new();
    let mut pid = std::process::id() as i32;
    for _ in 0..12 {
        let Some(ppid) = ps_ppid(pid) else {
            break;
        };
        if ppid <= 1 {
            break;
        }
        out.push(serde_json::json!({
            "pid": ppid,
            "ppid": ps_ppid(ppid),
            "comm": ps_field(ppid, "comm="),
            "args": ps_field(ppid, "args="),
        }));
        pid = ppid;
    }
    out
}

fn ps_ppid(pid: i32) -> Option<i32> {
    ps_field(pid, "ppid=")?.trim().parse().ok()
}

fn ps_field(pid: i32, field: &str) -> Option<String> {
    std::process::Command::new("ps")
        .args(["-o", field, "-p", &pid.to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn redacted_env() -> BTreeMap<String, String> {
    std::env::vars()
        .map(|(key, value)| {
            let value = if is_sensitive_env(&key) {
                format!("<redacted:{} bytes>", value.len())
            } else {
                value
            };
            (key, value)
        })
        .collect()
}

fn is_sensitive_env(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("api_key")
        || key.contains("apikey")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("passwd")
        || key.contains("credential")
        || key.contains("private")
        || key.contains("bearer")
        || key.contains("nsec")
        || (key.contains("auth") && key != "ssh_auth_sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_log_path_is_opt_in() {
        assert_eq!(configured_log_path_from(None), None);
        assert_eq!(configured_log_path_from(Some("")), None);
        assert_eq!(configured_log_path_from(Some("off")), None);
        assert_eq!(
            configured_log_path_from(Some("/tmp/command-calls.jsonl")),
            Some(PathBuf::from("/tmp/command-calls.jsonl"))
        );
    }
}
