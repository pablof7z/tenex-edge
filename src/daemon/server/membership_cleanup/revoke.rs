use super::*;
use std::collections::BTreeSet;

pub(in crate::daemon::server) fn recorded_channels(
    state: &Arc<DaemonState>,
    pubkey: &str,
) -> Vec<String> {
    state.with_store(|store| {
        let Some(session) = store.get_session(pubkey).ok().flatten() else {
            return Vec::new();
        };
        let mut channels = store
            .list_session_routes(&session.pubkey)
            .unwrap_or_default()
            .into_iter()
            .map(|(channel, _)| channel)
            .collect::<BTreeSet<_>>();
        if !session.channel_h.is_empty() {
            channels.insert(session.channel_h.clone());
        }
        channels.extend(
            store
                .list_session_standing(&session.pubkey)
                .unwrap_or_default()
                .into_iter()
                .filter(|standing| standing.state != crate::state::StandingState::Absent)
                .map(|standing| standing.channel_h),
        );
        channels.into_iter().collect()
    })
}

/// Explicit operator destruction has no grace window. Attempt every recorded
/// channel even when the local membership mirror is stale, and await read-back.
pub(in crate::daemon::server) async fn remove_revoked_session_memberships(
    state: &Arc<DaemonState>,
    pubkey: &str,
    channels: Vec<String>,
) -> Vec<String> {
    let _lane = state.standing_sync.lock().await;
    let mut failures = Vec::new();
    for channel in channels {
        let standing = state
            .with_store(|store| store.get_session_standing(pubkey, &channel))
            .ok()
            .flatten();
        let outcome = state
            .provider
            .remove_member_confirmed(&channel, pubkey)
            .await;
        if !outcome.is_confirmed() {
            failures.push(format!("{channel}: {outcome:?}"));
        } else if let Some(standing) = standing {
            if let Err(error) = state.with_store(|store| {
                store.mark_session_standing_absent_if_epoch(
                    pubkey,
                    &channel,
                    standing.state,
                    standing.standing_epoch,
                    standing.session_lifecycle_epoch,
                    now_secs(),
                )
            }) {
                failures.push(format!(
                    "{channel}: confirmed removal persistence: {error:#}"
                ));
            }
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::RegisterSession;

    #[tokio::test]
    async fn targets_recorded_channels_when_membership_cache_is_empty() {
        let state = DaemonState::new_for_test().await;
        let session = "pk-operator-kill";
        state.with_store(|store| {
            store
                .reserve_session(&RegisterSession {
                    pubkey: session.into(),
                    harness: "claude".into(),
                    agent_slug: "reviewer".into(),
                    channel_h: "active".into(),
                    child_pid: None,
                    transcript_path: None,
                    now: now_secs(),
                })
                .unwrap()
        });
        state
            .with_store(|store| store.grant_session_route(session, "joined", now_secs()))
            .unwrap();

        assert!(!state
            .with_store(|store| store.is_channel_member("active", "pk-operator-kill"))
            .unwrap());
        assert_eq!(
            recorded_channels(&state, session),
            vec![String::from("active"), String::from("joined")]
        );
    }
}
