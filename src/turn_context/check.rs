use crate::fabric_context::{inbox_seed, render_fabric_context, FabricContextInput};
use crate::state::{Session, Store};

use super::reads::{ambient_by_joined_channel, context_instance, joined_channels, take_inbox};

/// Mid-turn context for the PostToolUse `turn_check` hook.
pub(crate) fn assemble_turn_check_context(
    store: &std::sync::Mutex<Store>,
    rec: &Session,
    self_host: &str,
    delta_since: Option<u64>,
    now: u64,
) -> Option<String> {
    let mut warnings: Vec<String> = Vec::new();
    let scope = rec.channel_h.clone();
    let self_instance = context_instance(store, rec);
    let self_slug = self_instance.display_slug();
    let self_pubkey = self_instance.pubkey;
    let (joined, joined_read_failed) = {
        let s = store.lock().expect("store mutex poisoned");
        joined_channels(&s, rec)
    };

    let mut read_failed = joined_read_failed;
    let direct_mentions = {
        let s = store.lock().expect("store mutex poisoned");
        match take_inbox(&s, &rec.session_id, now) {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!(
                    session = %rec.session_id,
                    error = ?e,
                    "turn_check: inbox claim failed; direct mentions may be dropped"
                );
                read_failed = true;
                Vec::new()
            }
        }
    };
    if let Some(since) = delta_since {
        let s = store.lock().expect("store mutex poisoned");
        let (_ambient, ambient_failed) =
            ambient_by_joined_channel(&s, &joined, since, &self_pubkey);
        read_failed |= ambient_failed;
    }

    if read_failed {
        warnings.push(
            "Fabric read failed mid-turn; mentions and/or channel activity below \
             may be incomplete."
                .to_string(),
        );
    }

    if delta_since.is_none() && direct_mentions.is_empty() && warnings.is_empty() {
        return None;
    }
    let forced = direct_mentions.iter().map(inbox_seed).collect::<Vec<_>>();
    let cursor = delta_since.unwrap_or(now);
    let s = store.lock().expect("store mutex poisoned");
    render_fabric_context(
        &s,
        FabricContextInput {
            session: Some(rec),
            scope: &scope,
            cursor,
            now,
            self_slug: &self_slug,
            self_pubkey: &self_pubkey,
            local_host: self_host,
            edge_home: Some(&crate::config::edge_home()),
            forced_messages: &forced,
            warnings: &warnings,
            force: false,
        },
    )
}
