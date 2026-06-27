use crate::daemon::server::DaemonState;
use crate::tmux::pane::{pane_alive, paste_text, send_enter, tmux_available};
use crate::util::now_secs;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// How long to wait after `session_start` fires before typing into the pane.
/// The hook fires early in harness startup; we need to wait until the input
/// box is actually interactive.
const SPAWN_PROMPT_DELAY_MS: u64 = 2000;

/// Don't re-inject into the same session within this window (seconds).
const MESSAGE_INJECT_DEBOUNCE_SECS: u64 = 20;

static DEBOUNCE: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
fn debounce() -> &'static Mutex<HashMap<String, u64>> {
    DEBOUNCE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_debounced(session_id: &str) -> bool {
    let m = debounce().lock().unwrap();
    m.get(session_id)
        .map(|&t| now_secs().saturating_sub(t) < MESSAGE_INJECT_DEBOUNCE_SECS)
        .unwrap_or(false)
}

fn record_message_injection(session_id: &str) {
    debounce()
        .lock()
        .unwrap()
        .insert(session_id.to_string(), now_secs());
}

/// Type the received message into `pane_id` and submit it, so a freshly-spawned
/// harness opens on the message that triggered its spawn.
pub async fn inject_spawn_message(pane_id: &str, text: &str) -> Result<()> {
    tokio::time::sleep(Duration::from_millis(SPAWN_PROMPT_DELAY_MS)).await;
    if pane_alive(pane_id).is_none() {
        anyhow::bail!("pane {pane_id} died before spawn message could be injected");
    }

    paste_text(pane_id, text).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    send_enter(pane_id).await?;
    Ok(())
}

struct PendingTmuxPrompt {
    text: String,
    chat_ids: Vec<String>,
}

async fn collect_pending_prompt(
    state: &Arc<DaemonState>,
    rec: &crate::state::SessionRecord,
) -> Result<Option<PendingTmuxPrompt>> {
    let mut chat_rows = state.with_store(|s| s.peek_chat_mentions(&rec.session_id))?;
    if chat_rows.is_empty() {
        return Ok(None);
    }
    crate::profile::label_chat_senders(state, &mut chat_rows).await;

    let now = now_secs();
    let Some(text) = crate::injection::render_direct_mention_prompt(&chat_rows, now) else {
        return Ok(None);
    };

    Ok(Some(PendingTmuxPrompt {
        text,
        chat_ids: chat_rows
            .iter()
            .map(|row| row.chat_event_id.clone())
            .collect(),
    }))
}

fn mark_prompt_delivered(
    state: &Arc<DaemonState>,
    rec: &crate::state::SessionRecord,
    prompt: &PendingTmuxPrompt,
) -> Result<()> {
    let delivered_at = now_secs();
    state.with_store(|s| -> Result<()> {
        s.mark_chat_rows_delivered(&rec.session_id, &prompt.chat_ids, delivered_at)?;
        Ok(())
    })
}

/// Paste pending inbox/chat content into a live pane and submit it as the next
/// prompt. Returns false if another path consumed the rows before we injected.
pub async fn inject_pending_messages_pub(
    state: &Arc<DaemonState>,
    rec: &crate::state::SessionRecord,
    pane_id: &str,
) -> Result<bool> {
    let Some(prompt) = collect_pending_prompt(state, rec).await? else {
        return Ok(false);
    };

    paste_text(pane_id, &prompt.text).await?;
    mark_prompt_delivered(state, rec, &prompt)?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    send_enter(pane_id).await?;
    Ok(true)
}

/// Scans for sessions with unread inbox rows that have a live tmux endpoint,
/// and have not been injected recently.
pub fn ring_doorbells(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        if let Err(e) = ring_doorbells_inner(&state).await {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[tmux] pending message injection error: {e:#}");
            }
        }
    });
}

async fn ring_doorbells_inner(state: &Arc<DaemonState>) -> Result<()> {
    if !tmux_available() {
        return Ok(());
    }

    let sessions_with_chat: Vec<crate::state::SessionRecord> = state.with_store(|s| {
        s.list_alive_sessions()
            .unwrap_or_default()
            .into_iter()
            .filter(|rec| {
                s.count_unread_chat_mentions(&rec.session_id).unwrap_or(0) > 0
                    && !s.is_session_working(&rec.session_id)
            })
            .collect()
    });

    for rec in sessions_with_chat {
        let sid = rec.session_id.clone();
        if is_debounced(&sid) {
            continue;
        }

        let endpoint = state.with_store(|s| s.get_session_endpoint(&sid, "tmux"));
        let ep = match endpoint {
            Ok(Some(ep)) => ep,
            _ => continue,
        };

        let pane_id = ep.target.clone();
        if pane_alive(&pane_id).is_none() {
            if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                eprintln!("[tmux] pane {pane_id} gone; removing endpoint for {sid}");
            }
            state.with_store(|s| s.delete_session_endpoint(&sid, "tmux").ok());
            continue;
        }

        record_message_injection(&sid);
        let ts = now_secs();
        state.with_store(|s| s.touch_session_endpoint_verified(&sid, "tmux", ts).ok());

        match inject_pending_messages_pub(state, &rec, &pane_id).await {
            Ok(true) => {
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[tmux] pending messages injected into pane {pane_id} for session {sid}"
                    );
                }
            }
            Ok(false) => {}
            Err(e) => {
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[tmux] pending message inject failed for {sid} pane {pane_id}: {e:#}"
                    );
                }
            }
        }
    }
    Ok(())
}
