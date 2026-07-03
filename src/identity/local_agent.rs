use super::{atomic_write, key_path, validate_slug, StoredKey};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Set the local "when to use this agent" byline for an existing agent.
pub fn set_local_agent_byline(edge_home: &Path, slug: &str, byline: Option<String>) -> Result<()> {
    validate_slug(slug)?;
    let path = key_path(edge_home, slug);
    if !path.exists() {
        bail!("no such local agent: {slug}");
    }
    let s = std::fs::read_to_string(&path)
        .with_context(|| format!("reading key {}", path.display()))?;
    let mut stored: StoredKey =
        serde_json::from_str(&s).with_context(|| format!("parsing key {}", path.display()))?;
    stored.byline = byline
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let body = serde_json::to_string_pretty(&stored)?;
    atomic_write(&path, &body)?;
    Ok(())
}

/// Soft-delete the keystore file so a mistaken removal is recoverable.
pub fn remove_local_agent(edge_home: &Path, slug: &str) -> Result<Option<PathBuf>> {
    validate_slug(slug)?;
    let path = key_path(edge_home, slug);
    if !path.exists() {
        return Ok(None);
    }
    let parked = path.with_extension("json.removed");
    std::fs::rename(&path, &parked)
        .with_context(|| format!("parking {} -> {}", path.display(), parked.display()))?;
    Ok(Some(parked))
}
