//! Shared best-effort file logger for the distill path and the runtime engine.
//!
//! Both wrote near-identical `<edge_home>/logs/<file>` appenders with a localized
//! timestamp; this is the one implementation they share. Diagnostics only —
//! every failure (missing dir, unwritable file) is swallowed.

use std::io::Write;

/// Append one timestamped line to `<edge_home>/logs/<file>`. When `prefix` is
/// non-empty it is bracketed after the timestamp (`<ts> [<prefix>] <msg>`).
pub fn append(file: &str, prefix: &str, msg: &str) {
    let log_dir = crate::config::edge_home().join("logs");
    let _ = crate::config::ensure_dir(&log_dir);
    let path = log_dir.join(file);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let ts = crate::util::format_local_datetime_ms(crate::util::now_millis());
        if prefix.is_empty() {
            let _ = writeln!(f, "{ts} {msg}");
        } else {
            let _ = writeln!(f, "{ts} [{prefix}] {msg}");
        }
    }
}
