//! Harness-native resume-token creation and log extraction.

use crate::daemon::server::DaemonState;
use crate::session::Harness;
use crate::session_host::registry::HeadlessShape;
use crate::util::now_secs;
use anyhow::{Context, Result};
use std::io::Read as _;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn bind_native_id_from_log(
    state: &Arc<DaemonState>,
    pubkey: &str,
    harness: &str,
    log_path: &Path,
) {
    let Some(native_id) = extract_native_session_id(log_path) else {
        return;
    };
    if let Err(e) =
        state.with_store(|s| s.set_native_resume_locator(pubkey, harness, &native_id, now_secs()))
    {
        tracing::warn!(
            pubkey,
            harness,
            native_id,
            error = %e,
            "failed to bind native session id from headless log"
        );
    }
}

pub(super) fn harness_for_shape(shape: HeadlessShape) -> Harness {
    match shape {
        HeadlessShape::ClaudePrint => Harness::ClaudeCode,
        HeadlessShape::CodexExec => Harness::Codex,
        HeadlessShape::OpencodeRun => Harness::Opencode,
    }
}

pub(super) fn fresh_native_session_id(
    shape: HeadlessShape,
    resume_id: Option<&str>,
) -> Result<Option<String>> {
    match (shape, resume_id) {
        (HeadlessShape::ClaudePrint, None) => Ok(Some(random_uuid_v4()?)),
        _ => Ok(None),
    }
}

fn random_uuid_v4() -> Result<String> {
    let mut bytes = [0_u8; 16];
    std::fs::File::open("/dev/urandom")
        .context("opening /dev/urandom")?
        .read_exact(&mut bytes)
        .context("reading random UUID bytes")?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    ))
}

pub(crate) fn extract_native_session_id(log_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(log_path).ok()?;
    content.lines().find_map(|line| {
        let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
        native_id_from_value(&value)
    })
}

fn native_id_from_value(value: &serde_json::Value) -> Option<String> {
    if let Some(items) = value.as_array() {
        return items.iter().find_map(native_id_from_value);
    }
    const KEYS: &[&str] = &[
        "session_id",
        "sessionId",
        "sessionID",
        "conversation_id",
        "conversationId",
        "thread_id",
        "threadId",
    ];
    let object = value.as_object()?;
    for key in KEYS {
        if let Some(id) = object.get(*key).and_then(|v| v.as_str()) {
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    if let Some(id) = object
        .get("session")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .filter(|id| !id.is_empty())
    {
        return Some(id.to_string());
    }
    object.values().find_map(native_id_from_value)
}
