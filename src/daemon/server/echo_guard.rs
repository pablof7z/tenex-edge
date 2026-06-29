//! Echo suppression for tmux-pasted fabric envelopes.
//!
//! When the daemon pastes a mention into an agent's pane (the tmux delivery
//! path), the harness fires `user-prompt-submit` for that pasted text. But the
//! pasted message is ALREADY a kind:9 event in the room — republishing it would
//! echo it straight back into the channel (twice, on a publish retry).
//!
//! The old guard keyed on a visible `[tenex-edge]` text marker every envelope
//! had to start with. That was fragile (a human typing the marker got eaten)
//! and, worse, it dictated the envelope's first bytes — incompatible with the
//! bare/minimal injection forms. This guard moves the signal OFF the visible
//! text: the paste path records a hash of exactly what it pasted, and the
//! user-prompt publish path consumes the matching hash. Envelopes can now be any
//! shape — bare, minimal, or wrapped — without leaking a marker.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

/// How long a recorded injection stays matchable. Generous enough to cover the
/// harness round-trip (paste → render → `user-prompt-submit`), short enough that
/// a human later typing the identical text is mirrored normally.
const ECHO_TTL_SECS: u64 = 60;

/// Per-session ring of recently-pasted-text hashes. Self-locking so it can be a
/// plain (non-`Mutex`) field on `DaemonState`.
#[derive(Default)]
pub(crate) struct EchoGuard {
    /// `session_id` → `[(text_hash, recorded_at)]`.
    inner: Mutex<HashMap<String, Vec<(u64, u64)>>>,
}

fn hash_text(text: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.trim().hash(&mut h);
    h.finish()
}

impl EchoGuard {
    /// Record that `text` was just pasted into `session_id`'s pane, so the
    /// resulting `user-prompt-submit` is recognized as an echo and dropped.
    pub(crate) fn record(&self, session_id: &str, text: &str, now: u64) {
        let mut map = self.inner.lock().expect("echo guard poisoned");
        let entry = map.entry(session_id.to_string()).or_default();
        entry.retain(|(_, ts)| now.saturating_sub(*ts) < ECHO_TTL_SECS);
        entry.push((hash_text(text), now));
    }

    /// True iff `text` matches a fresh recorded injection for `session_id`. The
    /// match is CONSUMED so a genuine later repeat (a human typing the same
    /// words) is mirrored normally instead of being eaten.
    pub(crate) fn is_echo(&self, session_id: &str, text: &str, now: u64) -> bool {
        let mut map = self.inner.lock().expect("echo guard poisoned");
        let Some(entry) = map.get_mut(session_id) else {
            return false;
        };
        entry.retain(|(_, ts)| now.saturating_sub(*ts) < ECHO_TTL_SECS);
        let want = hash_text(text);
        if let Some(pos) = entry.iter().position(|(h, _)| *h == want) {
            entry.remove(pos);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_consumes_match() {
        let g = EchoGuard::default();
        g.record("s1", "  <@pablo> hi\n", 100);
        // Whitespace-insensitive match, and it is consumed on first hit.
        assert!(g.is_echo("s1", "<@pablo> hi", 110));
        assert!(!g.is_echo("s1", "<@pablo> hi", 111));
    }

    #[test]
    fn no_match_for_other_session_or_text() {
        let g = EchoGuard::default();
        g.record("s1", "hello", 100);
        assert!(!g.is_echo("s2", "hello", 101));
        assert!(!g.is_echo("s1", "different", 101));
    }

    #[test]
    fn expired_entries_do_not_match() {
        let g = EchoGuard::default();
        g.record("s1", "hello", 100);
        assert!(!g.is_echo("s1", "hello", 100 + ECHO_TTL_SECS + 1));
    }
}
