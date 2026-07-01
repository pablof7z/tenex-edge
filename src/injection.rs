//! Prompt rendering for fabric message injection.
//!
//! Four envelope forms, chosen by `(delivery method, sender, directedness)`:
//!
//!   1. **tmux + human mention** — pasted into the pane as a real turn, minimal
//!      provenance: `<@pablo> @developer hey there`. The agent's reply is
//!      auto-captured and published, so no reply instructions are needed.
//!   2. **tmux + agent mention** — pasted as a turn, framed so the agent knows it
//!      arrived via the fabric: `[tenex-edge mention] <@agent1> hi @developer`.
//!   3. **hook mention** — context the harness appends (never pasted, so it never
//!      echoes); wrapped in `<tenex-edge>…</tenex-edge>` with an explicit
//!      `chat write` reply hint, because a hooks-only agent must reply via CLI.
//!   4. **ambient chatter** — background channel activity, wrapped and presented
//!      as FYI; never forces a turn.
//!
//! Echo suppression no longer lives in this text (it moved to
//! [`crate::daemon::server`]'s per-session echo guard), so envelopes are free to
//! be bare. Message ids are intentionally absent: replies target `@name`.

use crate::state::{InboxRow, RelayEvent, Store};
use crate::util::{pubkey_short, relative_time};
use std::fmt::Write as _;

/// Below this age a mention is "fresh" and its relative-time suffix is omitted to
/// keep the line clean; ambient rows always show time (they are a timeline).
const STALE_SECS: u64 = 120;

/// When a speaker chip should carry a ` - <relative time>` suffix.
#[derive(Clone, Copy)]
enum TimePolicy {
    /// Live turn (tmux paste) — never timestamped.
    Never,
    /// Direct mention via hook — only when older than [`STALE_SECS`].
    WhenOld,
    /// Ambient timeline — always.
    Always,
}

/// `<@name>` or `<@name - 5 min ago>`, per `policy`.
fn speaker_chip(name: &str, created_at: u64, now: u64, policy: TimePolicy) -> String {
    let show = match policy {
        TimePolicy::Never => false,
        TimePolicy::Always => true,
        TimePolicy::WhenOld => now.saturating_sub(created_at) >= STALE_SECS,
    };
    if show {
        format!("<@{name} - {}>", relative_time(created_at, now))
    } else {
        format!("<@{name}>")
    }
}

/// Display name for a pubkey: its cached `kind:0` slug, else a short hex form.
fn speaker_label(store: &Store, pubkey: &str) -> String {
    store
        .resolve_slug_for_pubkey(pubkey)
        .ok()
        .flatten()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| pubkey_short(pubkey))
}

/// True when `pubkey` is one of the operator's whitelisted humans, i.e. the
/// mention came from a person rather than another agent.
fn is_whitelisted(whitelisted: &[String], pubkey: &str) -> bool {
    whitelisted.iter().any(|w| w.eq_ignore_ascii_case(pubkey))
}

/// Form ① / ② — direct mentions pasted into a live tmux pane as a real turn.
/// Human senders render bare with a `<@name>` prefix (it reads as a near-natural
/// turn that still carries provenance); agent senders are prefixed
/// `[tenex-edge mention]` so the agent knows it is a fabric relay, not its
/// operator typing. No reply hint, no message id — the reply auto-publishes.
pub(crate) fn render_tmux_mention(
    store: &Store,
    rows: &[InboxRow],
    whitelisted: &[String],
    now: u64,
) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = Vec::with_capacity(rows.len());
    for row in rows {
        let name = speaker_label(store, &row.from_pubkey);
        let chip = speaker_chip(&name, row.created_at, now, TimePolicy::Never);
        if is_whitelisted(whitelisted, &row.from_pubkey) {
            lines.push(format!("{chip} {}", row.body));
        } else {
            lines.push(format!("[tenex-edge mention] {chip} {}", row.body));
        }
    }
    Some(lines.join("\n"))
}

/// Form ③ — direct mentions delivered through a hook (no live pane). Wrapped and
/// carrying a `chat write` reply hint, because a hooks-only agent replies only by
/// calling the CLI. Time is shown only for stale rows.
pub(crate) fn render_hook_mention(
    store: &Store,
    channel_name: &str,
    rows: &[InboxRow],
    now: u64,
) -> Option<String> {
    let first = rows.first()?;
    let reply_to = speaker_label(store, &first.from_pubkey);
    let mut text = format!(
        "<tenex-edge>\nYou were mentioned in #{channel_name}. \
         Reply with: tenex-edge chat write \"@{reply_to} your reply\"\n"
    );
    for row in rows {
        let name = speaker_label(store, &row.from_pubkey);
        let chip = speaker_chip(&name, row.created_at, now, TimePolicy::WhenOld);
        let _ = write!(text, "\n{chip} {}", row.body);
    }
    text.push_str("\n</tenex-edge>");
    Some(text)
}

/// Form ④ — ambient channel chatter (not addressed to this agent). Wrapped FYI;
/// never forces a turn. `header` is the caller-built lead line (e.g.
/// `"Activity on #tenex-edge since you last looked:"`). Every row is timestamped.
pub(crate) fn render_ambient(
    store: &Store,
    header: &str,
    rows: &[RelayEvent],
    now: u64,
) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    let mut text = format!("<tenex-edge>\n{header}\n");
    for row in rows {
        let name = speaker_label(store, &row.pubkey);
        let chip = speaker_chip(&name, row.created_at, now, TimePolicy::Always);
        let content = crate::profile::rewrite_body_mentions(store, &row.content);
        let _ = write!(text, "\n{chip} {content}");
    }
    text.push_str("\n</tenex-edge>");
    Some(text)
}

/// The human display label for a channel: its kind:39000 `name` when set, else
/// the raw `channel_h` as a genuine fallback. The opaque id must never appear in
/// agent-facing text when a name exists.
pub(crate) fn channel_display(store: &Store, channel_h: &str) -> String {
    store
        .get_channel(channel_h)
        .ok()
        .flatten()
        .and_then(|c| c.human_name().map(str::to_string))
        .unwrap_or_else(|| channel_h.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_sdk::prelude::{Keys, ToBech32};

    #[test]
    fn render_ambient_rewrites_mention_entities_to_slugs() {
        let store = Store::open_memory().unwrap();
        let speaker = Keys::generate().public_key().to_hex();
        let mentioned = Keys::generate().public_key();
        store
            .upsert_profile(&mentioned.to_hex(), "Ada", "ada", "claude-code", false, 1)
            .unwrap();

        let row = RelayEvent {
            id: "e1".to_string(),
            kind: 9,
            pubkey: speaker,
            created_at: 1000,
            channel_h: "h1".to_string(),
            d_tag: String::new(),
            content: format!(
                "hey nostr:{} check this out",
                mentioned.to_bech32().unwrap()
            ),
            tags_json: "[]".to_string(),
        };

        let text = render_ambient(&store, "Activity:", &[row], 2000).unwrap();
        assert!(
            text.contains("@ada"),
            "mention entity should be rewritten to the resolved slug: {text}"
        );
        assert!(
            !text.contains("nostr:"),
            "raw nostr: entity must not leak into the rendered block: {text}"
        );
    }

    #[test]
    fn render_ambient_returns_none_for_no_rows() {
        let store = Store::open_memory().unwrap();
        assert!(render_ambient(&store, "Activity:", &[], 2000).is_none());
    }
}
