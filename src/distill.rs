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
//! the `edge-distillation` role in `~/.tenex/llms.json` called natively via
//! `rig` (openrouter/ollama) → `None`.

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

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

    fn run(&self, input: &str) -> Result<String> {
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
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string())
    }
}

/// System prompt for the native rig distiller.
const RIG_SYSTEM_PROMPT: &str = "Summarize what a coding agent is currently doing, in at most 8 words, present tense, intent not mechanics, no trailing punctuation. Output only the phrase.";

/// Summarize a context string into a one-line intent using rig (rig.rs), via
/// either openrouter or ollama per `resolved.provider`. Returns `None` on any
/// error (network, auth, empty output) so the caller can fall back.
pub async fn summarize_via_rig(
    resolved: &crate::llmconfig::ResolvedModel,
    context: &str,
) -> Option<String> {
    use rig::client::CompletionClient;
    use rig::completion::Prompt;

    let text: String = match resolved.provider.as_str() {
        "openrouter" => {
            let client = rig::providers::openrouter::Client::new(&resolved.api_key).ok()?;
            let agent = client
                .agent(&resolved.model)
                .preamble(RIG_SYSTEM_PROMPT)
                .temperature(0.2)
                .max_tokens(64)
                .build();
            let out: String = agent.prompt(context).await.ok()?;
            out
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
            let client = builder.build().ok()?;
            let agent = client
                .agent(&resolved.model)
                .preamble(RIG_SYSTEM_PROMPT)
                .temperature(0.2)
                .max_tokens(64)
                .build();
            let out: String = agent.prompt(context).await.ok()?;
            out
        }
        _ => return None,
    };

    let line = text.lines().map(str::trim).find(|l| !l.is_empty())?;
    let line: String = line.chars().take(80).collect();
    let line = line.trim().to_string();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

/// Distill the agent's current activity from its recent **conversation
/// transcript** (where real intent lives). Ordering:
///   (a) `$TENEX_EDGE_DISTILL_CMD` set → external command (explicit override);
///   (b) else the `edge-distillation` role resolves → native rig (openrouter/ollama);
///   (c) else `None` — LLM-only, no heuristic fallback.
pub async fn distill_activity(transcript: &str) -> Option<String> {
    let ctx = transcript.trim();
    if ctx.is_empty() {
        return None;
    }
    // (a) explicit external-command override.
    if let Some(cmd) = CommandDistiller::resolve() {
        if let Some(line) = cmd.summarize(ctx) {
            return Some(line);
        }
    }
    // (b) native rig via the edge-distillation role.
    if let Some(resolved) = crate::llmconfig::resolve_role("edge-distillation") {
        if let Some(line) = summarize_via_rig(&resolved, ctx).await {
            return Some(line);
        }
    }
    // (c) no fallback.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_distiller_uses_stdout_first_line() {
        let d = CommandDistiller {
            command: "cat >/dev/null; printf 'Fixing the auth bug\\nignored second line'".into(),
        };
        assert_eq!(
            d.summarize("User: fix the auth bug").unwrap(),
            "Fixing the auth bug"
        );
    }

    #[test]
    fn command_distiller_none_on_failure() {
        let d = CommandDistiller {
            command: "exit 1".into(),
        };
        assert!(d.summarize("anything").is_none());
    }

    #[test]
    fn command_distiller_none_on_empty_output() {
        let d = CommandDistiller {
            command: "cat >/dev/null; true".into(),
        };
        assert!(d.summarize("anything").is_none());
    }
}
