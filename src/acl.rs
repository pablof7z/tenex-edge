//! Owner-scoped agent authorization.
//!
//! Each computer keeps an **allowlist** of agent pubkeys it will see/trust
//! (`~/.tenex/whitelisted-agents.txt`) and a **blocklist**
//! (`~/.tenex/blocked-agents.txt`). Your own agents are auto-added to the
//! allowlist when their key is created. Foreign agents whose `kind:0` claims you
//! as owner (p-tags your pubkey) but aren't on either list are *pending* — the
//! human decides via `tenex-edge acl`.
//!
//! File format: one entry per line, `pubkey   # optional comment (slug)`.
//! Blank lines and `#`-comment lines are ignored.

use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::Path;

use crate::config;

/// Pubkeys this computer authorizes.
pub fn allowed() -> BTreeSet<String> {
    read_set(&config::agents_allowlist_path())
}

/// Pubkeys this computer has blocked.
pub fn blocked() -> BTreeSet<String> {
    read_set(&config::agents_blocklist_path())
}

pub fn is_allowed(pubkey: &str) -> bool {
    allowed().contains(pubkey)
}

pub fn is_blocked(pubkey: &str) -> bool {
    blocked().contains(pubkey)
}

/// Add a pubkey to the allowlist (idempotent). Removes it from the blocklist.
pub fn allow(pubkey: &str, comment: &str) -> Result<()> {
    remove_from(&config::agents_blocklist_path(), pubkey)?;
    add_to(&config::agents_allowlist_path(), pubkey, comment)
}

/// Add a pubkey to the blocklist (idempotent). Removes it from the allowlist.
pub fn block(pubkey: &str, comment: &str) -> Result<()> {
    remove_from(&config::agents_allowlist_path(), pubkey)?;
    add_to(&config::agents_blocklist_path(), pubkey, comment)
}

fn read_set(path: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Ok(s) = std::fs::read_to_string(path) {
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(tok) = line.split_whitespace().next() {
                out.insert(tok.to_string());
            }
        }
    }
    out
}

fn add_to(path: &Path, pubkey: &str, comment: &str) -> Result<()> {
    if read_set(path).contains(pubkey) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let line = if comment.trim().is_empty() {
        format!("{pubkey}\n")
    } else {
        format!("{pubkey}  # {}\n", comment.trim())
    };
    let mut existing = std::fs::read_to_string(path).unwrap_or_default();
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&line);
    std::fs::write(path, existing).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn remove_from(path: &Path, pubkey: &str) -> Result<()> {
    let Ok(s) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    let kept: Vec<&str> = s
        .lines()
        .filter(|line| line.split_whitespace().next() != Some(pubkey))
        .collect();
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    std::fs::write(path, out).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // These tests mutate process-global env vars; serialize them so parallel
    // runs don't clobber each other (or leak writes to the real ~/.tenex).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn isolate() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::env::set_var("TENEX_AGENTS_ALLOWLIST", d.path().join("allow.txt"));
        std::env::set_var("TENEX_AGENTS_BLOCKLIST", d.path().join("block.txt"));
        d
    }

    #[test]
    fn allow_block_roundtrip() {
        let _g = ENV_LOCK.lock().unwrap();
        let _d = isolate();
        assert!(!is_allowed("aa"));
        allow("aa", "coder").unwrap();
        allow("aa", "coder").unwrap(); // idempotent
        assert!(is_allowed("aa"));
        assert_eq!(allowed().len(), 1);

        // blocking moves it off the allowlist
        block("aa", "spam").unwrap();
        assert!(!is_allowed("aa"));
        assert!(is_blocked("aa"));

        // allowing again moves it back
        allow("aa", "coder").unwrap();
        assert!(is_allowed("aa"));
        assert!(!is_blocked("aa"));

        std::env::remove_var("TENEX_AGENTS_ALLOWLIST");
        std::env::remove_var("TENEX_AGENTS_BLOCKLIST");
    }

    #[test]
    fn parses_comments_and_blanks() {
        let _g = ENV_LOCK.lock().unwrap();
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.txt");
        std::fs::write(&p, "# header\n\nbb  # reviewer\ncc\n").unwrap();
        std::env::set_var("TENEX_AGENTS_ALLOWLIST", &p);
        let set = allowed();
        assert!(set.contains("bb") && set.contains("cc") && set.len() == 2);
        std::env::remove_var("TENEX_AGENTS_ALLOWLIST");
    }
}
