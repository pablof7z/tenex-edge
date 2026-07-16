use super::{scope::work_root_for, WhoRow, WhoSource};
use crate::state::{Session, Status, StoreReader};

/// Build a local-session row. Relay-confirmed agent-supplied status wins when present.
pub(super) fn local_row(store: StoreReader<'_>, s: &Session, local_host: &str, now: u64) -> WhoRow {
    let instance = local_instance(store, s);
    let live = store
        .get_status(&instance.pubkey, &s.channel_h)
        .ok()
        .flatten()
        .filter(|st| st.expiration == 0 || st.expiration >= now);
    let fresh = now.saturating_sub(s.last_seen) <= crate::session::STATUS_TTL_SECS;
    let state = crate::session_state::SessionState::classify(
        fresh,
        s.working,
        store.has_live_delivery_path(s),
    );
    let (title, activity) = live
        .map(|st| (st.title, live_activity(state.is_working(), st.activity)))
        .unwrap_or_else(|| (s.title.clone(), String::new()));
    let work_root = work_root_for(store, &s.channel_h);
    WhoRow {
        source: WhoSource::Local,
        state,
        slug: instance.display_slug(),
        channel: s.channel_h.clone(),
        status: title,
        activity,
        dormant: false,
        host: local_host.to_string(),
        age_secs: Some(now.saturating_sub(s.last_seen)),
        rel_cwd: String::new(),
        remote: false,
        work_root_display: work_root.clone(),
        work_root,
        pubkey: instance.pubkey,
    }
}

pub(super) fn local_instance(
    store: StoreReader<'_>,
    s: &Session,
) -> crate::identity::SessionIdentity {
    store
        .session_identity(&s.pubkey)
        .expect("session identity lookup failed")
        .expect("live session is missing its identity projection")
}

/// Build a peer row from relay-confirmed status; unknown host is treated local.
pub(super) fn peer_row(store: StoreReader<'_>, st: &Status, local_host: &str, now: u64) -> WhoRow {
    let host = store
        .get_profile(&st.pubkey)
        .ok()
        .flatten()
        .map(|p| p.host)
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| local_host.to_string());
    let work_root = work_root_for(store, &st.channel_h);
    let state = st.state.observed(st.expiration >= now);
    WhoRow {
        source: WhoSource::Peer,
        state,
        slug: peer_slug(store, st),
        channel: st.channel_h.clone(),
        status: st.title.clone(),
        activity: live_activity(state.is_working(), st.activity.clone()),
        dormant: false,
        remote: host.trim() != local_host,
        host,
        age_secs: Some(now.saturating_sub(st.last_seen)),
        rel_cwd: String::new(),
        work_root_display: work_root.clone(),
        work_root,
        pubkey: st.pubkey.clone(),
    }
}

pub(super) fn peer_slug(store: StoreReader<'_>, st: &Status) -> String {
    if !st.slug.is_empty() {
        return st.slug.clone();
    }
    store
        .resolve_slug_for_pubkey(&st.pubkey)
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::util::pubkey_short(&st.pubkey))
}

fn live_activity(active: bool, activity: String) -> String {
    if active {
        activity
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_peer_is_offline_without_stale_activity() {
        let store = crate::state::Store::open_memory().unwrap();
        let status = Status {
            pubkey: "peer".into(),
            channel_h: "root".into(),
            slug: "reviewer".into(),
            title: "Reviewing".into(),
            activity: "stale live activity".into(),
            state: crate::session_state::SessionState::Working,
            last_seen: 90,
            updated_at: 90,
            expiration: 100,
        };
        let row = peer_row(store.reader(), &status, "laptop", 101);
        assert_eq!(row.state, crate::session_state::SessionState::Offline);
        assert!(row.activity.is_empty());
    }
}
