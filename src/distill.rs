//! Activity distillation (M1 §8).
//!
//! The agent's recent **conversation transcript** is distilled into a one-line
//! intent that becomes its `Activity` note and live `Status`. This is **LLM-only**
//! (like `pc`) — intent isn't recoverable from tool calls by rule. There is no
//! heuristic fallback: if no model is configured (or the call fails), nothing is
//! published. The engine decides *when* to distill (30s into a turn, then
//! periodically) by watching turn state; this module only answers *what* the
//! agent is doing given the transcript.
//!
//! Ordering: `$TENEX_EDGE_DISTILL_CMD` (explicit external-command override) →
//! the `edge-distillation` role in `~/.tenex-edge/llms.json` dispatched by
//! provider: `claude-cli` → native `claude` CLI binary; `openrouter`/`ollama` →
//! native `rig` → `None`.

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

fn dlog(session_id: &str, msg: &str) {
    let log_dir = crate::config::edge_home().join("logs");
    let _ = crate::config::ensure_dir(&log_dir);
    let path = log_dir.join("distill.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let ts = crate::util::format_local_datetime_ms(ms);
        let short = &session_id[..8.min(session_id.len())];
        let _ = writeln!(f, "{ts} [{short}] {msg}");
    }
}

/// Pipes a context string to an external command's stdin; its stdout (first
/// non-empty line) is the distilled intent. This is the LLM seam — point
/// `$TENEX_EDGE_DISTILL_CMD` at any model CLI.
pub struct CommandDistiller {
    pub command: String,
}

impl CommandDistiller {
    /// Build from `$TENEX_EDGE_DISTILL_CMD`, if set.
    pub fn from_env() -> Option<Self> {
        std::env::var("TENEX_EDGE_DISTILL_CMD")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|command| Self { command })
    }

    /// Resolve the active distiller command: `$TENEX_EDGE_DISTILL_CMD` if set.
    /// This is an **explicit override** — the default distiller is the native rig
    /// path via the `edge-distillation` role (see [`distill_activity`]).
    pub fn resolve() -> Option<Self> {
        Self::from_env()
    }

    /// Summarize an arbitrary context string (e.g. a transcript snippet) into a
    /// one-line intent. Returns None on failure/empty.
    pub fn summarize(&self, context: &str) -> Option<String> {
        match self.run(context) {
            Ok(line) if !line.trim().is_empty() => Some(line),
            _ => None,
        }
    }

    /// Run the command and return its FULL stdout (all lines), so callers that
    /// need a multi-line response (e.g. the combined TITLE/NOW distiller) can
    /// parse it themselves. Returns None on spawn/exec failure.
    pub fn summarize_full(&self, context: &str) -> Option<String> {
        match self.run_all(context) {
            Ok(out) if !out.trim().is_empty() => Some(out),
            _ => None,
        }
    }

    fn run(&self, input: &str) -> Result<String> {
        Ok(self
            .run_all(input)?
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string())
    }

    fn run_all(&self, input: &str) -> Result<String> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }
        let out = child.wait_with_output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}

/// System prompt for the combined session distiller. ONE model call yields BOTH
/// a stable session TITLE (what the session is about) and a live NOW line (what
/// the agent is doing this moment) — see [`distill_session`]. Giving the model an
/// explicit NOW slot is what keeps step-level mechanics OUT of the title: it has
/// somewhere else to put them. The title is nudged-to-keep (only changes when the
/// objective substantively changes); NOW is regenerated every turn.
const SESSION_SYSTEM_PROMPT: &str = "You maintain two labels for a coding session. Output EXACTLY two lines, nothing else:\n\nTITLE: the session's overall objective — what the agent was asked to accomplish, NOT the step it happens to be doing right now. A stable noun phrase or imperative, at most 8 words, no trailing punctuation. Prefer the user's stated request. It must stay valid for the WHOLE session; if it would go stale in a few messages it is too specific — zoom out to the goal. You may be given the CURRENT title; if it still fits, repeat it verbatim. Only change it when the objective itself has substantively changed.\n\nNOW: what the agent is doing at this moment — the current step or mechanics. At most 8 words, present tense, no trailing punctuation. This is expected to change every turn.\n\nExample:\nTITLE: Fix GitHub issue 1\nNOW: reading the issue tracker";

/// One distilled (title, activity) pair from a single model call.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionLabels {
    /// Stable session title — what the session is about.
    pub title: String,
    /// Live "what it's doing now" line; empty when the model omits it.
    pub activity: String,
}

/// Trim a distilled label: strip whitespace and trailing sentence punctuation,
/// cap at 80 chars. Returns None when nothing meaningful remains.
fn clean_label(s: &str) -> Option<String> {
    let s = s.trim().trim_end_matches(['.', ' ', '\t']).trim();
    if s.is_empty() {
        return None;
    }
    let s: String = s.chars().take(80).collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Parse the two-line `TITLE:`/`NOW:` distiller output. Tolerant of label
/// casing, surrounding whitespace, and a model that emits only one of the lines.
/// Accepts `ACTIVITY`/`DOING` as synonyms for `NOW`. A bare single line with no
/// recognized label is treated as the title (degrade gracefully).
fn parse_labels(out: &str) -> (Option<String>, Option<String>) {
    let mut title = None;
    let mut activity = None;
    let mut unlabeled = None;
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line.split_once(':') {
            Some((label, value)) => match label.trim().to_ascii_uppercase().as_str() {
                "TITLE" => title = title.or_else(|| clean_label(value)),
                "NOW" | "ACTIVITY" | "DOING" => activity = activity.or_else(|| clean_label(value)),
                _ => unlabeled = unlabeled.or_else(|| clean_label(line)),
            },
            None => unlabeled = unlabeled.or_else(|| clean_label(line)),
        }
    }
    (title.or(unlabeled), activity)
}

/// Complete the combined session prompt natively via rig (openrouter/ollama).
/// Returns `Ok(Some(text))` on success, `Ok(None)` for unsupported provider or
/// empty output, `Err(msg)` when the LLM call itself fails so the caller can log it.
async fn complete_via_rig(
    resolved: &crate::llmconfig::ResolvedModel,
    context: &str,
) -> Result<Option<String>, String> {
    use rig::client::CompletionClient;
    use rig::completion::Prompt;

    let text: String = match resolved.provider.as_str() {
        "openrouter" => {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::HeaderName::from_static("http-referer"),
                reqwest::header::HeaderValue::from_static("https://github.com/pablof7z/tenex-edge"),
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("x-title"),
                reqwest::header::HeaderValue::from_static("tenex-edge"),
            );
            let client = rig::providers::openrouter::Client::builder()
                .api_key(&resolved.api_key)
                .http_headers(headers)
                .build()
                .map_err(|e| format!("openrouter client init: {e}"))?;
            let agent = client
                .agent(&resolved.model)
                .preamble(SESSION_SYSTEM_PROMPT)
                .temperature(0.2)
                .max_tokens(96)
                .build();
            agent
                .prompt(context)
                .await
                .map_err(|e| format!("openrouter/{} prompt failed: {e}", resolved.model))?
        }
        "ollama" => {
            use rig::providers::ollama::OllamaApiKey;
            // Ollama needs no auth by default; supply the empty key marker so the
            // builder's api-key type-state is satisfied (mirrors rig's own
            // `from_env`), then point it at the base URL from providers.json.
            let mut builder =
                rig::providers::ollama::Client::builder().api_key(OllamaApiKey::default());
            if !resolved.base_url.is_empty() {
                builder = builder.base_url(&resolved.base_url);
            }
            let client = builder
                .build()
                .map_err(|e| format!("ollama client init: {e}"))?;
            let agent = client
                .agent(&resolved.model)
                .preamble(SESSION_SYSTEM_PROMPT)
                .temperature(0.2)
                .max_tokens(96)
                .build();
            agent
                .prompt(context)
                .await
                .map_err(|e| format!("ollama/{} prompt failed: {e}", resolved.model))?
        }
        _ => return Ok(None),
    };
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Invoke the `claude` CLI binary in print-mode (`-p`) as the distillation LLM.
/// The system prompt is written to a fixed temp path (content is invariant);
/// the user context is piped to stdin. Returns the `result` field from the CLI's
/// JSON output, or `Err` on spawn/parse failure.
async fn complete_via_claude_cli(model: &str, context: &str) -> Result<Option<String>, String> {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;

    let mut child = tokio::process::Command::new("claude")
        .args([
            "-p",
            "--no-session-persistence",
            "--output-format",
            "json",
            "--disallowedTools",
            "*",
        ])
        .arg("--model")
        .arg(model)
        .arg("--system-prompt")
        .arg(SESSION_SYSTEM_PROMPT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn claude CLI: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(context.as_bytes()).await;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("claude CLI wait: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let v: serde_json::Value = serde_json::from_str(&stdout).map_err(|_| {
        format!(
            "parse claude CLI JSON; stdout=`{}` stderr=`{}`",
            stdout.chars().take(400).collect::<String>(),
            stderr.chars().take(400).collect::<String>(),
        )
    })?;

    if v.get("is_error").and_then(|x| x.as_bool()).unwrap_or(false) {
        let msg = v
            .get("result")
            .and_then(|x| x.as_str())
            .unwrap_or(stderr.as_str());
        return Err(format!("claude CLI error: {msg}"));
    }

    let text = v
        .get("result")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(if text.is_empty() { None } else { Some(text) })
}

/// Assemble a [`SessionLabels`] from parsed lines, applying nudge-to-keep: a
/// missing/empty title falls back to `current_title`. Returns None only when
/// there is no title at all (no parse, no current) — so the caller can try the
/// next provider.
fn assemble(
    parsed: (Option<String>, Option<String>),
    current_title: Option<&str>,
) -> Option<SessionLabels> {
    let (title, activity) = parsed;
    let title = title.or_else(|| {
        current_title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
    })?;
    Some(SessionLabels {
        title,
        activity: activity.unwrap_or_default(),
    })
}

/// Distill BOTH the persistent session title and the live activity line from the
/// recent transcript in a **single** model call (see [`SESSION_SYSTEM_PROMPT`]).
/// The `current_title` is fed back so the model keeps a still-accurate title
/// (nudge-to-keep). Ordering:
///   (a) `$TENEX_EDGE_DISTILL_CMD` set → external command (explicit override);
///   (b) else the `edge-distillation` role resolves:
///         – provider = `claude-cli` → native `claude` CLI binary;
///         – provider = `openrouter`/`ollama` → native rig;
///   (c) else **nudge-to-keep**: retain the current title with an empty activity.
///
/// Returns `(labels, error)`. `error` is `Some` only when the LLM was actually
/// called and failed — the caller should log it and surface it in the statusline.
/// A nudge-to-keep (no model configured, empty transcript) is not an error.
pub async fn distill_session(
    transcript: &str,
    current_title: Option<&str>,
    session_id: &str,
) -> (Option<SessionLabels>, Option<String>) {
    let ctx = transcript.trim();
    if ctx.is_empty() {
        // Nothing new to read: keep the title, no live activity.
        return (
            current_title
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(|t| SessionLabels {
                    title: t.to_string(),
                    activity: String::new(),
                }),
            None,
        );
    }
    // Give the model the current title as context so it can choose to keep it.
    let input = match current_title.map(str::trim).filter(|t| !t.is_empty()) {
        Some(t) => format!("CURRENT TITLE: {t}\n\nTRANSCRIPT:\n{ctx}"),
        None => ctx.to_string(),
    };
    // (a) explicit external-command override. The command sees SESSION_SYSTEM_PROMPT
    // semantics by convention (it is the LLM seam); we parse its two-line output.
    if let Some(cmd) = CommandDistiller::resolve() {
        dlog(session_id, "using TENEX_EDGE_DISTILL_CMD override");
        if let Some(out) = cmd.summarize_full(&input) {
            if let Some(labels) = assemble(parse_labels(&out), current_title) {
                return (Some(labels), None);
            }
        }
        dlog(
            session_id,
            "TENEX_EDGE_DISTILL_CMD produced no usable output",
        );
    }
    // (b) edge-distillation role — dispatch by provider.
    let mut rig_error: Option<String> = None;
    match crate::llmconfig::resolve_role("edge-distillation") {
        None => dlog(
            session_id,
            "edge-distillation role not resolved (check llms.json + providers.json)",
        ),
        Some(resolved) => {
            dlog(
                session_id,
                &format!("calling {}/{}", resolved.provider, resolved.model),
            );
            let result = match resolved.provider.as_str() {
                "claude-cli" => complete_via_claude_cli(&resolved.model, &input).await,
                _ => complete_via_rig(&resolved, &input).await,
            };
            match result {
                Ok(Some(out)) => {
                    dlog(session_id, &format!("distill response: {out:?}"));
                    if let Some(labels) = assemble(parse_labels(&out), current_title) {
                        return (Some(labels), None);
                    }
                    dlog(
                        session_id,
                        "parse/assemble produced no labels from response",
                    );
                }
                Ok(None) => dlog(
                    session_id,
                    "distiller returned empty output (unsupported provider or blank)",
                ),
                Err(e) => {
                    dlog(session_id, &format!("distill error: {e}"));
                    rig_error = Some(e);
                }
            }
        }
    }
    // (c) no model / empty output → keep the existing title (nudge-to-keep).
    dlog(
        session_id,
        &format!("falling back to nudge-to-keep current_title={current_title:?}"),
    );
    let labels = current_title
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| SessionLabels {
            title: t.to_string(),
            activity: String::new(),
        });
    (labels, rig_error)
}

#[cfg(test)]
mod tests;
