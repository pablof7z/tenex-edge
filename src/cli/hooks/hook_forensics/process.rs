use serde_json::Value;
use std::collections::HashMap;

pub(super) struct ProcessTreeSnapshot {
    rows: HashMap<i32, ProcessRow>,
}

struct ProcessRow {
    ppid: i32,
    args: String,
}

impl ProcessTreeSnapshot {
    /// Capture one coherent process-tree view. Each ancestor is queried once
    /// for all needed fields; a global `ps -ax` is slower on busy hosts.
    pub(super) fn capture(detailed: bool) -> Self {
        let mut rows = HashMap::new();
        let mut pid = std::process::id() as i32;
        rows.insert(
            pid,
            ProcessRow {
                ppid: unsafe { libc::getppid() },
                args: std::env::args().collect::<Vec<_>>().join(" "),
            },
        );
        if !detailed {
            return Self { rows };
        }
        for _ in 0..12 {
            let Some(row) = read_row(pid) else {
                break;
            };
            let ppid = row.ppid;
            rows.insert(pid, row);
            if ppid <= 1 {
                break;
            }
            pid = ppid;
        }
        Self { rows }
    }

    pub(super) fn current_process(&self) -> Value {
        let pid = std::process::id() as i32;
        serde_json::json!({
            "pid": pid,
            "ppid": self.rows.get(&pid).map(|row| row.ppid),
            "exe": std::env::current_exe().ok().map(|p| p.display().to_string()),
            "argv": std::env::args().collect::<Vec<_>>(),
            "cwd": std::env::current_dir().ok().map(|p| p.display().to_string()),
        })
    }

    pub(super) fn parent_chain(&self) -> Vec<Value> {
        let mut chain = Vec::new();
        let mut pid = std::process::id() as i32;
        for _ in 0..12 {
            let Some(parent) = self.rows.get(&pid) else {
                break;
            };
            let ppid = parent.ppid;
            if ppid <= 1 {
                break;
            }
            let Some(row) = self.rows.get(&ppid) else {
                break;
            };
            chain.push(serde_json::json!({
                "pid": ppid,
                "ppid": row.ppid,
                "comm": command_name(&row.args),
                "args": &row.args,
            }));
            pid = ppid;
        }
        chain
    }
}

fn read_row(pid: i32) -> Option<ProcessRow> {
    let output = std::process::Command::new("ps")
        .args(["-ww", "-o", "ppid=", "-o", "args=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let output = String::from_utf8(output.stdout).ok()?;
    let line = output.lines().next()?.trim_start();
    let split = line.find(char::is_whitespace)?;
    let ppid = line[..split].parse().ok()?;
    let args = line[split..].trim().to_string();
    Some(ProcessRow { ppid, args })
}

fn command_name(args: &str) -> Option<&str> {
    let command = args.split_whitespace().next()?;
    std::path::Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
}
