//! The per-session engine (M1 §5, §7).
//!
//! Runs as a daemon-hosted task. It is a thin driver over the local `sessions`
//! row (the canonical local process record). It:
//!   - publishes the agent's `kind:0` profile once,
//!   - feeds every kind:30315 status trigger (startup, TTL tick, distill, turn
//!     edge, channel change, session-end) into the ONE status reconciler
//!     ([`crate::reconcile::status`]), which decides when to (re)publish; the
//!     emitted effects are signed + parked on the `outbox` (the single executor),
//!   - schedules background distillation (`set_session_distill`; the title feeds
//!     kind:30315 only — it never renames the route channel),
//!   - watches the host PID and marks the session dead (`mark_dead`) on pid death
//!     or `cancel` (the `session-end` path).
//!
//! There is no per-session-room branching: the only channel distinction is
//! `parent` (`is_root_channel`/`channel_parent`), and management authority is
//! `is_channel_admin`, never a local owns-group flag.

use crate::distill;
use crate::domain::{DomainEvent, Profile, Status};
use crate::fabric::provider::Nip29Provider;
use crate::state::{Session, Store};
use crate::util::now_secs;
use anyhow::Result;
use nostr_sdk::prelude::{JsonUtil, Keys, NostrSigner};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

fn slog(session_id: &str, msg: &str) {
    let log_dir = crate::config::edge_home().join("logs");
    let _ = crate::config::ensure_dir(&log_dir);
    // Per-session debug log filename keyed by the raw canonical session id (an
    // internal correlation handle; canonical ids are filename-safe).
    let path = log_dir.join(format!("{session_id}.log"));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let ts = crate::util::format_local_datetime_ms(ms);
        let _ = writeln!(f, "{ts} {msg}");
    }
}

pub struct EngineParams {
    /// The session's ONE authoritative agent-instance identity (issue #98): base
    /// slug, selected pubkey, ordinal, and display label all in one value. Every
    /// publish this engine makes (kind:0, kind:9, kind:30315) derives its wire
    /// identity and signing key from this — never from parallel slug/pubkey/key
    /// fields with base-vs-ordinal fallback rules at the callsite.
    pub instance: crate::identity::AgentInstance,
    /// The agent's durable (ordinal-0, file-backed) keypair — the derivation root
    /// for this instance's signing keys via [`AgentInstance::signing_keys`].
    pub base_keys: Keys,
    pub project: String,
    pub session_id: String,
    pub host: String,
    /// Project-relative working directory (§8e), advertised on presence/status.
    pub rel_cwd: String,
    /// The human owner pubkey(s) — p-tagged on our profile + presence.
    pub owners: Vec<String>,
    pub relays: Vec<String>,
    pub watch_pid: Option<i32>,
    pub store_path: PathBuf,
    pub heartbeat: Duration,
    /// How often the engine polls turn state to decide whether to distill.
    pub obs_interval: Duration,
    pub status_ttl: Duration,
    /// Delay from turn-start to the first title distillation (default 30s) —
    /// short turns that finish before this never cost an LLM call.
    pub turn_first: Duration,
    /// Safety re-distillation interval WITHIN a single long-running turn that has
    /// no new user message (default 0 = disabled).
    pub turn_repeat: Duration,
}

impl EngineParams {
    /// Keys used to SIGN this session's live events: the base keys for ordinal 0,
    /// this instance's derived ordinal keys otherwise. The base-vs-ordinal choice
    /// lives in [`AgentInstance::signing_keys`], not here.
    fn signing_keys(&self) -> Keys {
        self.instance.signing_keys(&self.base_keys)
    }
}

fn status_channels(p: &EngineParams, store: &Mutex<Store>, session: &Session) -> Vec<String> {
    let mut channels = match store.lock() {
        Ok(g) => g
            .list_session_joined_channels(&session.session_id)
            .unwrap_or_default()
            .into_iter()
            .map(|(channel, _)| channel)
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };
    if !session.channel_h.is_empty() && !channels.iter().any(|c| c == &session.channel_h) {
        channels.push(session.channel_h.clone());
    }
    if channels.is_empty() && !p.project.is_empty() {
        channels.push(p.project.clone());
    }
    channels.sort();
    channels.dedup();
    if let Ok(g) = store.lock() {
        channels.retain(|channel| !g.is_archived_channel(channel).unwrap_or(false));
    }
    channels
}

/// Encode + sign the status and park the signed JSON on the `outbox`. The drainer
/// publishes it (and records the relay-confirmed event); the engine never talks to
/// the relay for status.
async fn enqueue_status(
    provider: &Nip29Provider,
    keys: &Keys,
    store: &Mutex<Store>,
    status: Status,
    now: u64,
) {
    let builder = match provider.encode(&DomainEvent::Status(status)) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "enqueue_status: encoding status event failed — skipping this heartbeat");
            return;
        }
    };
    let unsigned = builder.build(keys.public_key());
    let signed = match keys.sign_event(unsigned).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "enqueue_status: signing status event failed — skipping this heartbeat");
            return;
        }
    };
    let json = signed.as_json();
    match store.lock() {
        Ok(g) => {
            if let Err(e) = g.enqueue_outbox(&json, now) {
                tracing::error!(error = %e, "enqueue_status: enqueue_outbox failed — status not published this cycle");
            }
        }
        Err(_) => tracing::error!(
            "enqueue_status: store mutex poisoned — status not published this cycle"
        ),
    }
}

/// Sign + enqueue every status effect the reconciler emitted (Publish/Expire both
/// carry a ready `Status`). The reconciler DECIDES; the outbox drainer executes.
async fn apply_status_effects(
    effects: Vec<crate::reconcile::StatusEffect>,
    provider: &Nip29Provider,
    keys: &Keys,
    store: &Mutex<Store>,
) {
    for effect in effects {
        let status = match effect {
            crate::reconcile::StatusEffect::Publish { status, .. }
            | crate::reconcile::StatusEffect::Expire { status } => status,
        };
        enqueue_status(provider, keys, store, status, now_secs()).await;
    }
}

/// A session's joined-channel set (archived excluded) — the canonical h-tag input.
fn channel_set(
    p: &EngineParams,
    store: &Mutex<Store>,
    session: &Session,
) -> std::collections::BTreeSet<String> {
    status_channels(p, store, session).into_iter().collect()
}

/// Run one reconciler method under the shared lock (never held across `.await`)
/// and apply the effects it decided — the single status seam.
async fn drive(
    status: &Mutex<crate::reconcile::StatusReconciler>,
    provider: &Nip29Provider,
    keys: &Keys,
    store: &Mutex<Store>,
    f: impl FnOnce(
        &mut crate::reconcile::StatusReconciler,
    ) -> trellis_core::GraphResult<crate::reconcile::StatusOutcome>,
) {
    let effects = f(&mut status.lock().expect("status reconciler poisoned"))
        .map(|o| o.effects)
        .unwrap_or_default();
    apply_status_effects(effects, provider, keys, store).await;
}

// ── daemon-hosted session task (the relocated engine) ────────────────────────

/// Run the per-session engine INSIDE the daemon, using the SHARED relay
/// connection and the SHARED store (single writer). The daemon owns one union
/// subscription and demuxes incoming events centrally; this task only:
///   - publishes the profile once (signed with the agent's keys),
///   - heartbeats `last_seen` and enqueues a re-armed kind:30315 onto the outbox,
///   - distills turn activity → `sessions.title`/`activity` + an outbox status,
///   - watches the host pid and marks the session dead (title RETAINED) on pid
///     death or `cancel` (the `session-end` path).
///
/// Store access goes through the shared `Arc<Mutex<Store>>`; the guard is held
/// only across the synchronous rusqlite calls, NEVER across `.await`.
pub async fn run_session_in_daemon(
    p: EngineParams,
    provider: std::sync::Arc<Nip29Provider>,
    store: std::sync::Arc<Mutex<Store>>,
    cancel: std::sync::Arc<tokio::sync::Notify>,
    status: std::sync::Arc<Mutex<crate::reconcile::StatusReconciler>>,
) -> Result<()> {
    let owners = p.owners.clone();
    let signing_keys = p.signing_keys();
    let aref = p.instance.agent_ref();

    macro_rules! st {
        ($f:expr) => {{
            let g = store.lock().expect("store mutex poisoned");
            #[allow(clippy::redundant_closure_call)]
            ($f)(&*g)
        }};
    }

    let publish_de = |ev: DomainEvent| {
        let provider = provider.clone();
        let keys = signing_keys.clone();
        async move {
            if let Err(e) = provider.publish(&ev, &keys).await {
                tracing::error!(error = %format!("{e:#}"), "run_session_in_daemon: domain-event publish failed");
            }
        }
    };

    // Load the session row, distinguishing a genuine "no such session" (None) from
    // a store error (loud): a swallowed Err here silently skips the heartbeat/distill
    // cycle that depends on the row, masking DB corruption as an idle session.
    let load_session = |label: &str| -> Option<Session> {
        match st!(|s: &Store| s.get_session(&p.session_id)) {
            Ok(row) => row,
            Err(e) => {
                tracing::error!(session = %p.session_id, error = %e, "{label}: get_session failed — skipping this cycle");
                None
            }
        }
    };

    // Publish identity card signed with this session's own key: base key for
    // ordinal 0 ("haiku"), derived key for ordinal N ("haiku1", etc.).
    publish_de(DomainEvent::Profile(Profile {
        agent: aref.clone(),
        host: p.host.clone(),
        owners: owners.clone(),
        is_backend: false,
    }))
    .await;

    let turn_first = p.turn_first.as_secs();
    let turn_repeat = p.turn_repeat.as_secs();

    // Scheduling bookkeeping (not session status):
    //   - the in-flight distill task,
    //   - last_distill_attempt: wall-clock retry gate (success time lives in the
    //     session row's last_distill_at),
    //   - cur_turn_started / prev_working: edge detection against the session's
    //     working/turn_started_at columns.
    let mut distill_task: Option<
        tokio::task::JoinHandle<(Option<distill::SessionLabels>, Option<String>)>,
    > = None;
    let mut last_distill_attempt: u64 = 0;
    let mut cur_turn_started: u64 = 0;
    let mut prev_working = false;

    // Assert liveness immediately and arm the first status.
    if let Err(e) = st!(|s: &Store| s.touch_session(&p.session_id, now_secs())) {
        tracing::error!(session = %p.session_id, error = %e, "touch_session failed — liveness not bumped at startup");
    }
    if let Some(session) = load_session("startup-status") {
        let now = now_secs();
        // Seed the session in the ONE status authority (opening publish).
        let chans = channel_set(&p, &store, &session);
        drive(&status, &provider, &signing_keys, &store, |r| {
            r.on_session_started(
                &p.session_id,
                &p.host,
                &aref.slug,
                &aref.pubkey,
                &p.rel_cwd,
                chans,
                session.working,
                &session.title,
                &session.activity,
                now,
            )
        })
        .await;
    }

    let mut hb = tokio::time::interval(p.heartbeat);
    let mut obs = tokio::time::interval(p.obs_interval);

    loop {
        tokio::select! {
            _ = hb.tick() => {
                if let Some(pid) = p.watch_pid {
                    if !pid_alive(pid) { break; }
                }
                // Liveness re-arm: bump last_seen, then the reconciler's TTL tick
                // pushes the NIP-40 expiration forward. The SINGLE re-arm cadence —
                // subsumes the old heartbeat enqueue AND the deleted global timer.
                let now = now_secs();
                if let Err(e) = st!(|s: &Store| s.touch_session(&p.session_id, now)) {
                    tracing::error!(session = %p.session_id, error = %e, "touch_session failed — liveness not re-armed this beat");
                }
                drive(&status, &provider, &signing_keys, &store, |r| r.on_tick(&p.session_id, now)).await;
            }
            _ = obs.tick() => {
                let now = now_secs();

                // ── collect a finished background distillation ────────────
                if distill_task.as_ref().is_some_and(|h| h.is_finished()) {
                    let (result, error) = distill_task.take().unwrap().await.ok().unwrap_or((None, None));
                    slog(&p.session_id, &format!("[distill] task finished result={} error={:?}",
                        result.as_ref().map(|l| format!("title={:?} activity={:?}", l.title, l.activity)).unwrap_or_else(|| "None".into()),
                        error));
                    if let Some(labels) = result {
                        if let Err(e) = st!(|s: &Store| s.set_session_distill(
                            &p.session_id, &labels.title, &labels.activity, now,
                        )) {
                            tracing::error!(session = %p.session_id, error = %e, "set_session_distill failed — distilled title not persisted");
                        }
                        slog(&p.session_id, &format!("[distill] applied title={:?}", labels.title));

                        // The LLM output enters as a canonical input; the graph
                        // republishes iff title/activity changed (title → 30315 only).
                        drive(&status, &provider, &signing_keys, &store, |r| {
                            r.on_distill(&p.session_id, &labels.title, &labels.activity, now)
                        })
                        .await;
                    } else if let Some(err_msg) = error {
                        // Append to the per-session log for post-mortem inspection.
                        // (No DB error table in the new schema.)
                        slog(&p.session_id, &format!("[distill] ERROR: {err_msg}"));
                    }
                }

                let session = load_session("observe-tick");
                let (working, turn_started_at) = session
                    .as_ref()
                    .map(|s| (s.working, s.turn_started_at))
                    .unwrap_or((false, 0));

                // Feed observed turn state + channel set into the ONE authority; it
                // dedups (publishes only on a real busy/idle flip or channel change).
                if working != prev_working {
                    drive(&status, &provider, &signing_keys, &store, |r| {
                        if working { r.on_turn_start(&p.session_id, now) } else { r.on_turn_end(&p.session_id, now) }
                    })
                    .await;
                }
                if let Some(chans) = session.as_ref().map(|s| channel_set(&p, &store, s)) {
                    drive(&status, &provider, &signing_keys, &store, |r| {
                        r.on_channels_changed(&p.session_id, chans, now)
                    })
                    .await;
                }

                if working {
                    // ── rising edge / new user message ────────────────────
                    if turn_started_at != cur_turn_started {
                        cur_turn_started = turn_started_at;
                        // Fresh turn → reset distill scheduling.
                        last_distill_attempt = 0;
                        distill_task = None;
                    }

                    // ── schedule background distillation ──────────────────
                    if distill_task.is_none() {
                        if let Some(sess) = session.as_ref() {
                            let succeeded_this_turn =
                                sess.turn_started_at > 0 && sess.last_distill_at >= sess.turn_started_at;
                            let due = if last_distill_attempt == 0 {
                                now.saturating_sub(sess.turn_started_at) >= turn_first
                            } else if succeeded_this_turn {
                                turn_repeat > 0 && now.saturating_sub(sess.last_distill_at) >= turn_repeat
                            } else {
                                now.saturating_sub(last_distill_attempt) >= turn_first
                            };
                            if due {
                                let transcript_path = sess.transcript_path.clone();
                                slog(&p.session_id, &format!("[distill] due transcript_path={:?}", transcript_path));
                                let ctx = transcript_path.and_then(|path| {
                                    let result = crate::transcript::read_recent(std::path::Path::new(&path), 14, 2500);
                                    if result.is_none() {
                                        slog(&p.session_id, &format!("[distill] read_recent returned None path={path}"));
                                    }
                                    result
                                });
                                if let Some(ctx) = ctx {
                                    let current = (!sess.title.trim().is_empty())
                                        .then(|| sess.title.clone());
                                    slog(&p.session_id, &format!("[distill] spawning task ctx_len={} current_title={:?}", ctx.len(), current));
                                    last_distill_attempt = now;
                                    let sid = p.session_id.clone();
                                    distill_task = Some(tokio::spawn(async move {
                                        match tokio::time::timeout(
                                            Duration::from_secs(20),
                                            distill::distill_session(&ctx, current.as_deref(), &sid),
                                        )
                                        .await
                                        {
                                            Ok(pair) => pair,
                                            Err(_) => (None, Some("distillation timed out after 20s".to_string())),
                                        }
                                    }));
                                }
                            }
                        }
                    }
                } else if prev_working {
                    // Falling edge: turn closed. Reset local distill scheduling; the
                    // idle status was already published above via `on_turn_end`.
                    cur_turn_started = 0;
                    last_distill_attempt = 0;
                    distill_task = None;
                }
                prev_working = working;
            }
            _ = cancel.notified() => { break; }
        }
    }

    // Deterministic teardown: close the status scope → the reconciler emits a
    // FINAL, immediately-expiring kind:30315 (activity cleared, expiration = now)
    // so peers drop the session at once instead of waiting out the TTL.
    let end_now = now_secs();
    drive(&status, &provider, &signing_keys, &store, |r| {
        r.on_session_ended(&p.session_id, end_now)
    })
    .await;

    // Clean exit: mark the session dead (alive=0, working=0). The TITLE is retained
    // in the row; mention routing (list_alive_sessions) drops it immediately.
    if let Err(e) = st!(|s: &Store| s.mark_dead(&p.session_id)) {
        tracing::error!(session = %p.session_id, error = %e, "mark_dead failed — session row left alive after clean exit");
    }
    Ok(())
}

fn pid_alive(pid: i32) -> bool {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_pid_is_alive() {
        assert!(pid_alive(std::process::id() as i32));
    }
}
