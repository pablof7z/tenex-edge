//! Read final assistant output from a Claude Code (or compatible) transcript
//! JSONL for explicit chat auto-publishing.
//!
//! Transcript lines look like:
//!   {"type":"user","message":{"role":"user","content":"..."| [blocks]}, ...}
//!   {"type":"assistant","message":{"role":"assistant","content":[{type:text,text},{type:tool_use,name,input}]}}
//!   {"type":"response_item","payload":{"type":"message","role":"assistant","content":[{type:output_text,text}]}}
//! We extract assistant text while skipping tool-result noise.

use serde_json::Value;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const TAIL_BYTES: u64 = 96 * 1024;

/// The tail of `path` split into lines, dropping the partial first line left by
/// the mid-file seek. `None` if the file is unreadable.
fn tail_lines(path: &Path) -> Option<Vec<String>> {
    let mut f = File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let start = len.saturating_sub(TAIL_BYTES);
    f.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if start > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    Some(lines)
}

/// The text of the LAST assistant message in the transcript, capped at
/// `max_chars`. Used to auto-publish an agent's final response when its turn
/// ended without an explicit `channel send`. `None` when the transcript holds no
/// non-empty assistant text.
pub fn read_last_assistant_text(path: &Path, max_chars: usize) -> Option<String> {
    let lines = tail_lines(path)?;
    let mut last: Option<String> = None;
    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some((role, content)) = message_record(&v) else {
            continue;
        };
        if role != "assistant" {
            continue;
        }
        let body = extract(content, role);
        if !body.trim().is_empty() {
            last = Some(body.trim().to_string());
        }
    }
    last.map(|b| truncate(&b, max_chars))
}

fn message_record(v: &Value) -> Option<(&str, Option<&Value>)> {
    if v.get("type").and_then(|x| x.as_str()) == Some("response_item") {
        let payload = v.get("payload")?;
        if payload.get("type").and_then(|x| x.as_str()) != Some("message") {
            return None;
        }
        let role = payload.get("role").and_then(|x| x.as_str())?;
        if role != "user" && role != "assistant" {
            return None;
        }
        return Some((role, payload.get("content")));
    }

    if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
        if t != "user" && t != "assistant" {
            return None;
        }
        return Some((t, v.get("message").and_then(|m| m.get("content"))));
    }

    let role = v.get("role").and_then(|x| x.as_str())?;
    if role != "user" && role != "assistant" {
        return None;
    }
    Some((role, v.get("content")))
}

fn extract(content: Option<&Value>, _role: &str) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(blocks)) => {
            let mut parts = Vec::new();
            for b in blocks {
                // tool_use, tool_result, and other blocks are not reply text.
                if matches!(
                    b.get("type").and_then(|x| x.as_str()),
                    Some("text" | "input_text" | "output_text")
                ) {
                    if let Some(t) = b.get("text").and_then(|x| x.as_str()) {
                        parts.push(t.to_string());
                    }
                }
            }
            parts.join(" ")
        }
        _ => String::new(),
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect()
    }
}

#[cfg(test)]
mod tests;
