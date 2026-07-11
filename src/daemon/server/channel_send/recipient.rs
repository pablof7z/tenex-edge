use super::super::*;
use crate::state::Store;

pub(in crate::daemon::server) struct ResolvedRecipient {
    pub(super) pubkey: String,
    pub(super) target_session: Option<String>,
    pub(super) channel: String,
}

/// Resolve a recipient/identifier to a wire pubkey under the CANONICAL scheme:
///   - `agent@backend-label` → the durable agent on that backend (`@` NEVER
///     means channel). The message still goes to `my_channel`.
///   - 64-hex / npub → raw pubkey.
///   - a session     → by canonical id, harness alias, or id prefix (correlation
///     handles only; a session id is never a chat-target identity).
///   - a bare agent-instance label → that instance on the LOCAL host
///     (`label@<local_host>`), reverse-resolved to its selected pubkey.
///
/// Sessions are local-only in the new model (session ids never travel the wire),
/// so session-prefix matching searches the local `sessions` table; a remote agent
/// is addressed only by `agent@backend-label` or pubkey.
pub(in crate::daemon::server) fn resolve_recipient(
    store: &Store,
    my_channel: &str,
    local_host: &str,
    target: &str,
) -> Result<ResolvedRecipient> {
    use crate::idref::{parse_ref, Ref};

    let session_recipient =
        |store: &Store, session_id: String, fallback_pk: String, channel: String| {
            let pubkey = store
                .session_identity_for_session(&session_id)
                .ok()
                .flatten()
                .map(|i| i.pubkey)
                .or_else(|| {
                    store
                        .get_session(&session_id)
                        .ok()
                        .flatten()
                        .map(|s| s.agent_pubkey)
                })
                .unwrap_or(fallback_pk);
            ResolvedRecipient {
                pubkey,
                target_session: Some(session_id),
                channel,
            }
        };

    match parse_ref(target) {
        Ref::Agent { slug, host } => {
            let pk = store.resolve_agent_pubkey(&slug, &host)?.with_context(|| {
                format!("can't resolve {slug}@{host} (no profile seen yet — try `tenex-edge who`)")
            })?;
            Ok(ResolvedRecipient {
                pubkey: pk,
                target_session: None,
                channel: my_channel.to_string(),
            })
        }
        Ref::Pubkey(raw) => {
            let pubkey = nostr_sdk::prelude::PublicKey::parse(&raw)
                .map(|pk| pk.to_hex())
                .unwrap_or(raw);
            Ok(ResolvedRecipient {
                pubkey,
                target_session: None,
                channel: my_channel.to_string(),
            })
        }
        Ref::Token(tok) => {
            if let Some(pubkey) = store.pubkey_for_handle(&tok)? {
                if let Some(session) = store.session_for_pubkey(&pubkey)? {
                    return Ok(session_recipient(
                        store,
                        session.session_id,
                        pubkey,
                        session.channel_h,
                    ));
                }
            }
            if let Some(pubkey) =
                store.resolve_live_profile_handle_pubkey(&tok, crate::util::now_secs())?
            {
                return Ok(ResolvedRecipient {
                    pubkey,
                    target_session: None,
                    channel: my_channel.to_string(),
                });
            }
            // Bare agent-instance label → that instance on the LOCAL host
            //    (profile fallback for remote/snapshotted peers).
            if let Some(pk) = store.resolve_agent_pubkey(&tok, local_host.trim())? {
                return Ok(ResolvedRecipient {
                    pubkey: pk,
                    target_session: None,
                    channel: my_channel.to_string(),
                });
            }
            anyhow::bail!("can't resolve recipient {target:?} (try `tenex-edge who`)")
        }
    }
}
