//! Public local-session selection by authoritative pubkey or leased handle.

use crate::state::{Session, Store};
use anyhow::Result;

/// Resolve only user-facing identity forms. Runtime ids and locator aliases are
/// deliberately excluded; callers that own a typed runtime locator resolve it
/// through the lifecycle path that owns that locator.
pub(in crate::daemon::server) fn resolve(store: &Store, selector: &str) -> Result<Option<Session>> {
    let selector = selector.trim().trim_start_matches('@');
    let pubkey = match crate::idref::normalize_pubkey(selector) {
        Some(pubkey) => Some(pubkey),
        None => store.pubkey_for_handle(selector)?,
    };
    let Some(pubkey) = pubkey else {
        return Ok(None);
    };
    store.session_for_pubkey(&pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Identity, RegisterSession};
    use nostr_sdk::prelude::Keys;

    #[test]
    fn resolves_npub_hex_and_handle_but_not_private_runtime_id() {
        let store = Store::open_memory().unwrap();
        let pubkey = Keys::generate().public_key().to_hex();
        let run_id = store
            .register_session(&RegisterSession {
                harness: "codex".into(),
                external_id_kind: "harness_session".into(),
                external_id: "native-1".into(),
                agent_pubkey: pubkey.clone(),
                agent_slug: "codex".into(),
                channel_h: "root".into(),
                child_pid: None,
                transcript_path: None,
                resume_id: String::new(),
                now: 1,
            })
            .unwrap();
        store
            .upsert_identity(&Identity {
                pubkey: pubkey.clone(),
                agent_slug: "codex".into(),
                codename: String::new(),
                session_id: run_id.clone(),
                channel_h: "root".into(),
                native_id: "native-1".into(),
                alive: true,
                created_at: 1,
            })
            .unwrap();
        let handle = store.allocate_handle(&pubkey, "codex", 1).unwrap().handle;
        let npub = crate::idref::npub(&pubkey).unwrap();
        let mentioned_handle = format!("@{handle}");

        for selector in [&pubkey, &npub, &handle, &mentioned_handle] {
            assert_eq!(
                resolve(&store, selector).unwrap().unwrap().session_id,
                run_id
            );
        }
        assert!(resolve(&store, &run_id).unwrap().is_none());
        assert!(resolve(&store, &run_id[..8]).unwrap().is_none());
    }
}
