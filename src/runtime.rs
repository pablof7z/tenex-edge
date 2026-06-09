//! The per-session engine (M1 §5, §7).
//!
//! Runs in the detached background process forked by `session-start`. It:
//!   - publishes the agent's `kind:0` profile once,
//!   - heartbeats presence on an interval,
//!   - drains observed tool activity, distills it, publishes Activity + Status,
//!   - subscribes to the project + mentions-to-me, updating the peer directory
//!     and routing mentions into the per-session inbox,
//!   - watches the host PID and stops cleanly (idle status) when it dies or on
//!     SIGTERM (the `session-end` path).

use crate::codec::{Codec, Kind1Codec, SubScope};
use crate::distill;
use crate::domain::{Activity, AgentRef, DomainEvent, Mention, Presence, Profile, Status};
use crate::state::{InboxRow, Store};
use crate::transport::Transport;
use crate::util::now_secs;
use anyhow::Result;
use nostr_sdk::prelude::{
    Alphabet, Event, Filter, Kind, PublicKey, RelayMessage, RelayPoolNotification, SingleLetterTag,
};
use std::path::PathBuf;
use std::time::Duration;

pub struct EngineParams {
    pub agent_slug: String,
    pub agent_pubkey: String,
    pub keys: nostr_sdk::prelude::Keys,
    pub project: String,
    pub session_id: String,
    pub host: String,
    /// The human owner pubkey(s) — p-tagged on our profile + presence, and used
    /// to discover foreign agents claiming the same owner (ACL pending set).
    pub owners: Vec<String>,
    pub relays: Vec<String>,
    pub watch_pid: Option<i32>,
    pub store_path: PathBuf,
    pub heartbeat: Duration,
    /// How often the engine polls turn state to decide whether to distill.
    pub obs_interval: Duration,
    pub status_ttl: Duration,
    /// Delay from turn-start to the first activity distillation (default 30s) —
    /// short turns that finish before this never cost an LLM call.
    pub turn_first: Duration,
    /// Interval between subsequent distillations while a turn keeps running
    /// (default 5m), to refresh intent on long turns without spamming the LLM.
    pub turn_repeat: Duration,
}

/// Targets for an incoming mention addressed to me: a specific session if pinned
/// (and it is one of mine), else all my alive sessions for this agent.
pub fn compute_targets(target_session: Option<&str>, my_alive_sessions: &[String]) -> Vec<String> {
    match target_session {
        Some(ts) => {
            if my_alive_sessions.iter().any(|s| s == ts) {
                vec![ts.to_string()]
            } else {
                Vec::new()
            }
        }
        None => my_alive_sessions.to_vec(),
    }
}

pub async fn run_session(p: EngineParams) -> Result<()> {
    let store = Store::open(&p.store_path)?;
    let codec = Kind1Codec;
    let transport = Transport::connect(&p.relays, p.keys.clone()).await?;
    let me = p.agent_pubkey.clone();
    let aref = AgentRef::new(me.clone(), p.agent_slug.clone());
    let ttl = p.status_ttl.as_secs();

    let owners = p.owners.clone();
    let build_scope = |authors: Vec<String>| SubScope {
        authors,
        project: Some(p.project.clone()),
        mentions_to: Some(me.clone()),
        owners: owners.clone(),
    };

    // Publish our identity card (declares our owner(s) via p-tags).
    publish(
        &transport,
        &codec,
        DomainEvent::Profile(Profile {
            agent: aref.clone(),
            host: p.host.clone(),
            owners: owners.clone(),
        }),
    )
    .await;

    // Subscribe: trusted authors (the ACL allowlist ∪ me) + this project +
    // mentions to me + owner-discovery. Trust is recomputed each heartbeat and
    // re-subscribed when it changes (e.g. after `tenex-edge acl allow`).
    let mut current_authors = trusted_authors(&me);
    let mut notifications = transport.notifications();
    let initial_filters = codec.filters(&build_scope(current_authors.clone()));
    if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
        eprintln!(
            "[engine] connected; subscribing {} filters; owners={:?} authors={:?}",
            initial_filters.len(),
            owners,
            current_authors
        );
    }
    transport.subscribe(initial_filters).await?;

    // One-shot fetch of kind 39000 (NIP-29 group metadata) for this project.
    // Populates the project_meta cache so `who` can show about-text for other
    // projects without waiting for a live publish.
    let group_meta_filter = Filter::new()
        .kind(Kind::from(39000u16))
        .custom_tag(
            SingleLetterTag::lowercase(Alphabet::D),
            p.project.as_str(),
        );
    if let Ok(events) = transport
        .fetch(group_meta_filter, Duration::from_secs(2))
        .await
    {
        for ev in events {
            if let Some(project) = event_tag(&ev, "d") {
                let about = event_tag(&ev, "about").unwrap_or("");
                store
                    .upsert_project_meta(project, about, ev.created_at.as_secs())
                    .ok();
            }
        }
    }

    // Announce liveness immediately (don't make presence wait on anything).
    let presence = |expires_at| {
        DomainEvent::Presence(Presence {
            agent: aref.clone(),
            project: p.project.clone(),
            session_id: p.session_id.clone(),
            host: p.host.clone(),
            audience: owners.clone(),
            expires_at,
        })
    };
    publish(&transport, &codec, presence(now_secs() + ttl)).await;
    publish_status(&transport, &codec, &aref, &p.project, "", ttl).await; // start idle
    store.set_agent_status(&me, &p.project, "", now_secs()).ok();
    store.touch_session(&p.session_id, now_secs()).ok(); // mark myself live now

    // Startup: pull recent stored mentions addressed to me (offline delivery).
    if let Ok(pk) = PublicKey::from_hex(&me) {
        let f = Filter::new()
            .kind(nostr_sdk::prelude::Kind::from(1u16))
            .pubkey(pk)
            .limit(50);
        if let Ok(events) = transport.fetch(f, Duration::from_secs(2)).await {
            for ev in events {
                if let Some(DomainEvent::Mention(m)) = codec.decode(&ev) {
                    route_mention(&store, &me, &m, &ev);
                }
            }
        }
    }

    // Turn-driven activity. The host's turn-start/turn-end hooks flip `turn_state`;
    // we poll it. The LLM distiller fires `turn_first` (30s) into a turn, then
    // every `turn_repeat` (5m) while it runs — so a turn that finishes in <30s
    // never costs a call. Engine-local state tracks the current turn + the last
    // line we published, so the heartbeat can refresh the (TTL'd) Status cheaply
    // without re-running the LLM.
    let turn_first = p.turn_first.as_secs();
    let turn_repeat = p.turn_repeat.as_secs();
    let mut cur_turn_start: u64 = 0; // turn we're tracking (0 = idle)
    let mut last_distill: u64 = 0; // when we last attempted a distill this turn
    let mut cur_line: Option<String> = None; // last published intent (None = idle)

    let mut hb = tokio::time::interval(p.heartbeat);
    let mut obs = tokio::time::interval(p.obs_interval);
    let mut sigterm = unix_sigterm();

    loop {
        tokio::select! {
            _ = hb.tick() => {
                if let Some(pid) = p.watch_pid {
                    if !pid_alive(pid) { break; }
                }
                // Pick up newly-authorized agents (allowlist changes).
                let latest = trusted_authors(&me);
                if latest != current_authors {
                    current_authors = latest;
                    let _ = transport.subscribe(codec.filters(&build_scope(current_authors.clone()))).await;
                }
                // Housekeeping: drop peers whose heartbeats stopped long ago.
                let _ = store.prune_peer_sessions(now_secs().saturating_sub(PRUNE_PEER_AFTER_SECS));
                store.touch_session(&p.session_id, now_secs()).ok(); // keep myself fresh
                publish(&transport, &codec, presence(now_secs() + ttl)).await;
                // Refresh the (TTL'd) Status while a turn is live, so a long turn
                // between distillations doesn't let our status expire. This re-
                // publishes the replaceable kind:30315 only — NOT the append-only
                // Activity note — and runs no LLM. The heartbeat (30s) is well
                // under the status TTL (90s).
                if let Some(line) = cur_line.clone() {
                    publish_status(&transport, &codec, &aref, &p.project, &line, ttl).await;
                    store.set_agent_status(&me, &p.project, &line, now_secs()).ok();
                }
            }
            _ = obs.tick() => {
                let (working, turn_started_at) = store.get_turn_state(&p.session_id).unwrap_or((false, 0));
                let now = now_secs();
                if working {
                    // A fresh turn_started_at means a new turn: reset the distill clock.
                    if turn_started_at != cur_turn_start {
                        cur_turn_start = turn_started_at;
                        last_distill = 0;
                    }
                    // First distill `turn_first` into the turn, then every `turn_repeat`.
                    let due = if last_distill == 0 {
                        now.saturating_sub(cur_turn_start) >= turn_first
                    } else {
                        now.saturating_sub(last_distill) >= turn_repeat
                    };
                    if due {
                        // Distill from the conversation transcript (LLM-only). On a
                        // missing transcript or LLM failure nothing publishes — by
                        // design — but we still advance `last_distill` so a failing
                        // model isn't retried every tick (we respect the cadence).
                        let ctx = store
                            .get_session_transcript(&p.session_id)
                            .ok()
                            .flatten()
                            .and_then(|path| crate::transcript::read_recent(std::path::Path::new(&path), 14, 2500));
                        if let Some(ctx) = ctx {
                            if let Some(line) = distill::distill_activity(&ctx).await {
                                publish(&transport, &codec, DomainEvent::Activity(Activity {
                                    agent: aref.clone(),
                                    project: p.project.clone(),
                                    text: format!("{line} #{}", p.project),
                                })).await;
                                publish_status(&transport, &codec, &aref, &p.project, &line, ttl).await;
                                store.set_agent_status(&me, &p.project, &line, now).ok();
                                cur_line = Some(line);
                            }
                        }
                        last_distill = now;
                    }
                } else {
                    // Turn ended (or never started): go idle, once.
                    if cur_line.is_some() {
                        publish_status(&transport, &codec, &aref, &p.project, "", ttl).await;
                        store.set_agent_status(&me, &p.project, "", now).ok();
                        cur_line = None;
                    }
                    cur_turn_start = 0;
                    last_distill = 0;
                }
            }
            n = notifications.recv() => {
                // Handle both the deduped `Event` variant and the raw `Message`
                // (EVENT) variant — auth-gated relays + the warmup fetch can mark
                // events "already seen", suppressing the `Event` variant.
                let ev: Option<Event> = match n {
                    Ok(RelayPoolNotification::Event { event, .. }) => Some(*event),
                    Ok(RelayPoolNotification::Message {
                        message: RelayMessage::Event { event, .. }, ..
                    }) => Some(event.into_owned()),
                    Ok(_) => None,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(_) => None, // lagged; keep going
                };
                if let Some(event) = ev {
                    handle_incoming(&store, &me, &owners, &codec, &event);
                }
            }
            _ = recv_sigterm(&mut sigterm) => { break; }
        }
    }

    // Clean exit: go idle and mark the session dead.
    publish(&transport, &codec, presence(now_secs())).await;
    publish_status(&transport, &codec, &aref, &p.project, "", ttl).await;
    store.mark_session_dead(&p.session_id).ok();
    transport.shutdown().await;
    Ok(())
}

fn handle_incoming(store: &Store, me: &str, owners: &[String], codec: &Kind1Codec, event: &Event) {
    // NIP-29 group metadata: cache the 'about' text for the channel.
    if event.kind.as_u16() == 39000 {
        if let Some(project) = event_tag(event, "d") {
            let about = event_tag(event, "about").unwrap_or("");
            store
                .upsert_project_meta(project, about, event.created_at.as_secs())
                .ok();
        }
        return;
    }

    let is_self = event.pubkey.to_hex() == me;
    let Some(de) = codec.decode(event) else {
        return;
    };
    let now = now_secs();
    if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
        let kind = event.kind.as_u16();
        let author = &event.pubkey.to_hex()[..8];
        eprintln!(
            "[recv] kind={kind} author={author} variant={}",
            de_name(&de)
        );
    }
    match de {
        // Our own profile/presence/activity/status are noise to us — skip. But a
        // Mention to our own pubkey (a sibling session of the same agent) must
        // still be routed, so it falls through below.
        DomainEvent::Profile(_)
        | DomainEvent::Presence(_)
        | DomainEvent::Activity(_)
        | DomainEvent::Status(_)
            if is_self => {}
        DomainEvent::Profile(pf) => {
            let pk = &pf.agent.pubkey;
            if crate::acl::is_allowed(pk) {
                // Authorized agent: into the directory; clear any pending entry.
                store.upsert_profile(pk, &pf.agent.slug, &pf.host, now).ok();
                store.remove_pending_agent(pk).ok();
            } else if !crate::acl::is_blocked(pk) && pf.owners.iter().any(|o| owners.contains(o)) {
                // Unknown agent claiming our owner → pending human decision.
                store
                    .upsert_pending_agent(pk, &pf.agent.slug, &pf.host, &pf.owners.join(","), now)
                    .ok();
            }
        }
        DomainEvent::Presence(pr) => {
            if pr.expires_at <= now {
                return;
            }
            store
                .upsert_peer_session(
                    &pr.session_id,
                    &pr.agent.pubkey,
                    &pr.agent.slug,
                    &pr.project,
                    &pr.host,
                    now,
                )
                .ok();
            if !pr.agent.slug.is_empty() {
                store
                    .upsert_profile(&pr.agent.pubkey, &pr.agent.slug, &pr.host, now)
                    .ok();
            }
        }
        DomainEvent::Status(st) => {
            if st.expires_at.map(|e| e <= now).unwrap_or(false) {
                return;
            }
            // What a peer is currently doing (self is handled above).
            store
                .set_agent_status(&st.agent.pubkey, &st.project, &st.text, now)
                .ok();
        }
        DomainEvent::Mention(m) if m.to_pubkey == me => {
            route_mention(store, me, &m, event);
        }
        _ => {}
    }
}

fn route_mention(store: &Store, me: &str, m: &Mention, event: &Event) {
    // Already delivered to this agent in some session? Don't re-enqueue it in a
    // new session (mentions persist on the relay as stored kind:1 events).
    let eid = event.id.to_hex();
    if store.is_mention_seen(me, &eid).unwrap_or(false) {
        return;
    }
    let alive: Vec<String> = store
        .list_alive_sessions()
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s.agent_pubkey == me)
        .map(|s| s.session_id)
        .collect();
    let targets = compute_targets(m.target_session.as_deref(), &alive);
    for t in targets {
        let _ = store.enqueue_mention(&InboxRow {
            mention_event_id: event.id.to_hex(),
            target_session: t,
            from_pubkey: m.from.pubkey.clone(),
            from_slug: m.from.slug.clone(),
            project: m.project.clone(),
            body: m.body.clone(),
            created_at: now_secs(),
        });
    }
}

async fn publish(transport: &Transport, codec: &Kind1Codec, ev: DomainEvent) {
    if let Ok(b) = codec.encode(&ev) {
        let _ = transport.publish_builder(b).await;
    }
}

async fn publish_status(
    transport: &Transport,
    codec: &Kind1Codec,
    agent: &AgentRef,
    project: &str,
    text: &str,
    ttl: u64,
) {
    let ev = DomainEvent::Status(Status {
        agent: agent.clone(),
        project: project.to_string(),
        text: text.to_string(),
        expires_at: Some(now_secs() + ttl),
    });
    publish(transport, codec, ev).await;
}

/// Drop peers whose heartbeat stopped more than this ago (pollution cleanup).
const PRUNE_PEER_AFTER_SECS: u64 = 600;

/// Trusted authors = the ACL allowlist ∪ me, sorted & unique. The allowlist is
/// the set of agent pubkeys this computer has authorized (own fleet is
/// auto-added on key creation; foreign agents via `tenex-edge acl`).
fn trusted_authors(me: &str) -> Vec<String> {
    let mut set: Vec<String> = crate::acl::allowed().into_iter().collect();
    set.push(me.to_string());
    set.sort();
    set.dedup();
    set
}

fn de_name(de: &DomainEvent) -> &'static str {
    match de {
        DomainEvent::Profile(_) => "Profile",
        DomainEvent::Presence(_) => "Presence",
        DomainEvent::Activity(_) => "Activity",
        DomainEvent::Status(_) => "Status",
        DomainEvent::Mention(_) => "Mention",
    }
}

fn event_tag<'a>(event: &'a Event, name: &str) -> Option<&'a str> {
    event.tags.iter().find_map(|t| {
        let s = t.as_slice();
        if s.first().map(String::as_str) == Some(name) {
            s.get(1).map(String::as_str)
        } else {
            None
        }
    })
}

fn pid_alive(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

// ── SIGTERM handling (the graceful session-end path) ─────────────────────────

#[cfg(unix)]
type SigTerm = tokio::signal::unix::Signal;
#[cfg(unix)]
fn unix_sigterm() -> Option<SigTerm> {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok()
}
#[cfg(unix)]
async fn recv_sigterm(s: &mut Option<SigTerm>) {
    match s {
        Some(sig) => {
            sig.recv().await;
        }
        None => std::future::pending::<()>().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_pinned_session_only_if_mine() {
        let mine = vec!["s1".to_string(), "s2".to_string()];
        assert_eq!(compute_targets(Some("s2"), &mine), vec!["s2"]);
        assert!(compute_targets(Some("not-mine"), &mine).is_empty());
    }

    #[test]
    fn targets_agent_level_fans_out_to_all_my_sessions() {
        let mine = vec!["s1".to_string(), "s2".to_string()];
        assert_eq!(compute_targets(None, &mine), mine);
    }

    #[test]
    fn current_pid_is_alive() {
        assert!(pid_alive(std::process::id() as i32));
    }
}
