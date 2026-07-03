//! Small shared helpers for the interactive `inquire` flows: turning a
//! canceled prompt (Esc/Ctrl-C) into "go back a menu" instead of an error,
//! and masking secrets for display.

use anyhow::Result;
use inquire::InquireError;

/// Convert a prompt result into `Ok(None)` when the user canceled (Esc or
/// Ctrl-C) instead of propagating it as an error — callers pair this with
/// `let Some(x) = prompted(...)? else { return Ok(()) };` to back out one
/// menu level on cancel.
pub(super) fn prompted<T>(r: std::result::Result<T, InquireError>) -> Result<Option<T>> {
    match r {
        Ok(v) => Ok(Some(v)),
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Mask a secret for display as `head…tail` (e.g. `sk-or-…4f2a`) — enough of
/// the prefix to recognize which key it is, never enough to reconstruct it.
/// Values too short to mask usefully are replaced entirely.
pub(super) fn mask_secret(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 9 {
        return "…".to_string();
    }
    let head: String = chars[..5].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}…{tail}")
}
