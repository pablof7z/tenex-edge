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
            // 1. Public `sessionCode-agent` handle. Do this before raw session
            // matching so a malformed handle never falls through to a bare role.
            if crate::idref::parse_session_handle(&tok).is_some() {
                if let Some(found) = find_session_by_public_handle(store, my_channel, &tok)? {
                    return Ok(session_recipient(
                        store,
                        found.session_id,
                        found.pubkey,
                        found.channel,
                    ));
                }
                if let Some(pk) = store.resolve_profile_handle_pubkey(&tok)? {
                    return Ok(ResolvedRecipient {
                        pubkey: pk,
                        target_session: None,
                        channel: my_channel.to_string(),
                    });
                }
                anyhow::bail!("can't resolve recipient {target:?} (try `tenex-edge who`)");
            }

            // 2. Exact canonical id or harness alias.
            if let Some(s) = store.get_session(&tok)? {
                return Ok(session_recipient(
                    store,
                    s.session_id,
                    s.agent_pubkey,
                    s.channel_h,
                ));
            }
            // 3. Local session id prefix. A store error here must NOT collapse into
            // "no such recipient" — propagate it so a DB failure is loud, not a
            // silent unknown-mention.
            if tok.len() >= 6 {
                if let Some(s) = store
                    .list_alive_sessions()
                    .context("resolve_recipient: listing live sessions for id-prefix match")?
                    .into_iter()
                    .find(|s| s.session_id.starts_with(&tok))
                {
                    return Ok(session_recipient(
                        store,
                        s.session_id,
                        s.agent_pubkey,
                        s.channel_h,
                    ));
                }
            }
            if crate::idref::looks_like_agent_first_session_handle(&tok) {
                anyhow::bail!(
                    "can't resolve recipient {target:?}: session handles use @sessionCode-agent"
                );
            }
            // 4. Bare agent-instance label → that instance on the LOCAL host
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

#[derive(Clone)]
struct SessionMatch {
    pubkey: String,
    session_id: String,
    channel: String,
}

fn find_session_by_public_handle(
    store: &Store,
    my_channel: &str,
    handle: &str,
) -> Result<Option<SessionMatch>> {
    let Some((agent_slug, session_ref)) = crate::idref::parse_session_handle(handle) else {
        return Ok(None);
    };
    let agent_slug = agent_slug.to_ascii_lowercase();
    let session_ref = session_ref.to_ascii_lowercase();
    let matches = candidate_sessions(store, my_channel)?
        .into_iter()
        .filter(|(session, instance)| {
            session.agent_slug.to_ascii_lowercase() == agent_slug
                || instance.slug.to_ascii_lowercase() == agent_slug
        })
        .filter(|(session, instance)| {
            let session_id = session.session_id.to_ascii_lowercase();
            let codename = instance.codename.to_ascii_lowercase();
            let short = crate::util::friendly_short_code(&session.session_id).to_ascii_lowercase();
            session_id == session_ref
                || codename == session_ref
                || short == session_ref
                || (session_ref.len() >= 6 && session_id.starts_with(&session_ref))
        })
        .map(|(session, instance)| session_match(store, my_channel, session, instance))
        .collect::<Vec<_>>();
    choose_unique_session_label_match(handle, "all channels", matches)
}

fn candidate_sessions(
    store: &Store,
    context: &str,
) -> Result<Vec<(crate::state::Session, crate::identity::SessionIdentity)>> {
    let sessions = store
        .list_alive_sessions()
        .with_context(|| format!("{context}: listing live sessions"))?;
    Ok(sessions
        .into_iter()
        .map(|session| {
            let instance = store
                .session_identity_for_session(&session.session_id)
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    crate::identity::SessionIdentity::fallback(
                        &session.session_id,
                        session.agent_slug.clone(),
                        session.agent_pubkey.clone(),
                    )
                });
            (session, instance)
        })
        .collect())
}

fn session_match(
    store: &Store,
    my_channel: &str,
    session: crate::state::Session,
    instance: crate::identity::SessionIdentity,
) -> SessionMatch {
    let joined_current = store
        .is_session_joined_channel(&session.session_id, my_channel)
        .unwrap_or(session.channel_h == my_channel);
    let channel = if joined_current {
        my_channel.to_string()
    } else {
        session.channel_h.clone()
    };
    SessionMatch {
        pubkey: instance.pubkey,
        session_id: session.session_id,
        channel,
    }
}

fn choose_unique_session_label_match(
    label: &str,
    scope: &str,
    mut matches: Vec<SessionMatch>,
) -> Result<Option<SessionMatch>> {
    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.pop()),
        _ => anyhow::bail!(
            "agent label @{label} matches multiple live sessions in {scope}; run `tenex-edge who`"
        ),
    }
}
