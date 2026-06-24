//! Shared prompt rendering for fabric message injection.
//!
//! Tmux delivery submits direct mentions as a real harness prompt. Hook fallback
//! can only emit through each host's hook context shape, so the text itself makes
//! the role explicit and stays byte-identical across delivery paths.

use crate::state::ChatInboxRow;
use crate::util::{format_local_datetime, pubkey_short, relative_time};
use std::fmt::Write as _;

/// Prefix every fabric-injected prompt carries. The daemon pastes these envelopes
/// into a pane as a real harness prompt; the resulting `user-prompt-submit` hook
/// must NOT republish them (they are already kind:9 events in the room). A real
/// human prompt never starts with this marker, so it is a reliable discriminator.
pub(crate) const FABRIC_INJECTION_MARKER: &str = "[tenex-edge]";

/// True when `prompt` is a daemon-injected fabric envelope rather than human
/// keyboard input — i.e. content that is already published and must not be
/// mirrored back into the room by the user-prompt publish path.
pub(crate) fn is_fabric_injection(prompt: &str) -> bool {
    prompt.trim_start().starts_with(FABRIC_INJECTION_MARKER)
}

pub(crate) fn split_direct_mentions(
    rows: Vec<ChatInboxRow>,
    self_session: &str,
) -> (Vec<ChatInboxRow>, Vec<ChatInboxRow>) {
    rows.into_iter()
        .partition(|row| row.mentioned_session == self_session)
}

pub(crate) fn render_direct_mention_prompt(rows: &[ChatInboxRow], now: u64) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    let noun = if rows.len() == 1 {
        "message"
    } else {
        "messages"
    };
    // Sender-agnostic preamble: a mention may originate from a human OR another
    // agent, so the envelope must not assert "user-authored".
    let mut text = format!(
        "{FABRIC_INJECTION_MARKER} Incoming {noun} mentioning this agent. \
         Treat the following as input addressed to you in this session:"
    );
    append_rows_with_kind(&mut text, rows, now, RowKind::DirectMention);
    Some(text)
}

pub(crate) fn render_channel_chat_block(
    header: &str,
    rows: &[ChatInboxRow],
    now: u64,
) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    let mut text = String::from(header);
    append_rows_with_kind(&mut text, rows, now, RowKind::ChannelContext);
    Some(text)
}

enum RowKind {
    DirectMention,
    ChannelContext,
}

fn append_rows_with_kind(text: &mut String, rows: &[ChatInboxRow], now: u64, kind: RowKind) {
    for row in rows {
        let from = if row.from_slug.is_empty() {
            pubkey_short(&row.from_pubkey)
        } else {
            row.from_slug.clone()
        };
        // Sender-agnostic wording: a mention may come from a human OR another
        // agent, so never assume "user". A direct mention reads "Mention in
        // #channel from <sender>"; sibling channel context stays "Channel
        // message from <sender>".
        let label = match kind {
            RowKind::DirectMention => format!("Mention in {}", channel_label(&row.project)),
            RowKind::ChannelContext => {
                format!("Channel message in {}", channel_label(&row.project))
            }
        };
        let _ = write!(
            text,
            "\n\n{} from {} at {} ({})\n{}",
            label,
            from,
            format_local_datetime(row.created_at),
            relative_time(row.created_at, now),
            row.body
        );
        if !row.chat_event_id.is_empty() {
            let _ = write!(text, "\n(message id: {})", pubkey_short(&row.chat_event_id));
        }
    }
}

fn channel_label(project: &str) -> String {
    if project.starts_with('#') {
        project.to_string()
    } else {
        format!("#{project}")
    }
}
