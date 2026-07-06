use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchMetadata {
    pub id: String,
    pub socket: String,
    pub supervisor_pid: u32,
    pub agent: String,
    pub project: String,
    pub cwd: String,
    pub command: Vec<String>,
}

pub fn session_dir() -> PathBuf {
    crate::config::edge_home().join("pty")
}

pub fn session_socket(id: &str) -> PathBuf {
    session_dir().join(format!("{id}.sock"))
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

pub fn resolve_socket(id_or_path: &str) -> PathBuf {
    let path = PathBuf::from(id_or_path);
    if path.components().count() > 1 || id_or_path.ends_with(".sock") {
        path
    } else {
        session_socket(id_or_path)
    }
}
