use crate::daemon::server::DaemonState;
use crate::util::now_secs;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

#[path = "delivery/output_mode.rs"]
mod output_mode;
mod prompt;
use crate::session_host::transport::{
    transport_kind_for_slug, AcpTransport, EndpointRef, SessionTransport, TransportKind,
};
#[cfg(test)]
pub(crate) use output_mode::headless_for_endpoint;
pub(crate) use output_mode::session_is_headless;
use prompt::{inject_planned_messages_acp, inject_planned_messages_pty};

#[cfg(test)]
#[path = "delivery/tests.rs"]
mod delivery_tests;

fn locator_kind(kind: TransportKind) -> &'static str {
    match kind {
        TransportKind::Pty => crate::state::LOCATOR_PTY,
        TransportKind::Acp => crate::state::LOCATOR_ACP,
    }
}

fn endpoint_id_for(
    store: &crate::state::Store,
    pubkey: &str,
    kind: TransportKind,
) -> Result<Option<String>> {
    Ok(store
        .locators_for_pubkey(pubkey)?
        .into_iter()
        .find(|locator| locator.locator_kind == locator_kind(kind))
        .map(|locator| locator.locator_value))
}

/// Liveness of a session's typed transport endpoint.
fn endpoint_is_live(kind: TransportKind, endpoint_id: &str) -> bool {
    match kind {
        TransportKind::Pty => crate::pty::is_live(endpoint_id),
        TransportKind::Acp => AcpTransport.is_live(&EndpointRef {
            kind: TransportKind::Acp,
            endpoint_id: endpoint_id.to_string(),
        }),
    }
}

/// Whether `session` has a live daemon-owned delivery endpoint.
pub(crate) fn session_has_live_delivery_endpoint(
    store: &crate::state::Store,
    session: &crate::state::Session,
) -> bool {
    let kind = transport_kind_for_slug(&session.agent_slug);
    let endpoint_id = match endpoint_id_for(store, &session.pubkey, kind) {
        Ok(endpoint_id) => endpoint_id,
        Err(e) => {
            tracing::error!(
                pubkey = %session.pubkey,
                error = %e,
                "delivery endpoint check: locator lookup failed; assuming unavailable"
            );
            return false;
        }
    };
    endpoint_id.is_some_and(|endpoint_id| endpoint_is_live(kind, &endpoint_id))
}

/// How long to wait after `session_start` fires before typing into the PTY.
/// The hook fires early in harness startup; we need to wait until the input
/// box is actually interactive.
const SPAWN_PROMPT_DELAY_MS: u64 = 2000;

/// Don't re-inject into the same session within this window (seconds).
const MESSAGE_INJECT_DEBOUNCE_SECS: u64 = 20;

static DEBOUNCE: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
fn debounce() -> &'static Mutex<HashMap<String, u64>> {
    DEBOUNCE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn last_message_injection(pubkey: &str) -> Option<u64> {
    debounce().lock().unwrap().get(pubkey).copied()
}

fn record_message_injection(pubkey: &str) {
    debounce()
        .lock()
        .unwrap()
        .insert(pubkey.to_string(), now_secs());
}

fn prune_debounce(active_pubkeys: &HashSet<String>) {
    let now = now_secs();
    debounce().lock().unwrap().retain(|pubkey, last| {
        active_pubkeys.contains(pubkey) && now.saturating_sub(*last) < MESSAGE_INJECT_DEBOUNCE_SECS
    });
}

/// Type the received message into `endpoint_id` and submit it, so a freshly-spawned
/// harness opens on the message that triggered its spawn.
pub async fn inject_spawn_message(endpoint_id: &str, text: &str) -> Result<()> {
    tokio::time::sleep(Duration::from_millis(SPAWN_PROMPT_DELAY_MS)).await;
    if !crate::pty::is_live(endpoint_id) {
        anyhow::bail!("pty session {endpoint_id} died before spawn message could be injected");
    }

    crate::pty::inject(endpoint_id, text, true, true)?;
    Ok(())
}

/// Deliver a fresh session's opening prompt over whichever transport hosts it:
/// ACP via a JSON-RPC `deliver` (submit=true → a fresh turn), PTY via the
/// bracketed-paste spawn inject. The ACP child lives in the daemon registry, so
/// this MUST run in the daemon (the caller that spawned it). Failures are logged,
/// not propagated: the session is already live and can still receive mentions via
/// the doorbell path.
pub async fn deliver_spawn_prompt(agent_slug: &str, endpoint_id: &str, text: &str) {
    match transport_kind_for_slug(agent_slug) {
        TransportKind::Acp => {
            let ep = EndpointRef {
                kind: TransportKind::Acp,
                endpoint_id: endpoint_id.to_string(),
            };
            if let Err(e) = AcpTransport.deliver(&ep, text, true).await {
                tracing::warn!(
                    agent = %agent_slug,
                    endpoint = %endpoint_id,
                    error = %format!("{e:#}"),
                    "failed to deliver ACP spawn prompt"
                );
            }
        }
        TransportKind::Pty => {
            if let Err(e) = inject_spawn_message(endpoint_id, text).await {
                tracing::warn!(
                    agent = %agent_slug,
                    endpoint = %endpoint_id,
                    error = %format!("{e:#}"),
                    "failed to inject PTY spawn prompt"
                );
            }
        }
    }
}

pub async fn inject_pending_messages_pty(
    state: &Arc<DaemonState>,
    rec: &crate::state::Session,
    endpoint_id: &str,
) -> Result<bool> {
    let pending = state.with_store(|s| s.peek_pending_for_pubkey(&rec.pubkey))?;
    if pending.is_empty() {
        return Ok(false);
    };
    let fact = delivery_scan_fact(
        rec,
        pending.iter().map(|row| row.event_id.clone()).collect(),
        Some(endpoint_id.to_string()),
        crate::pty::is_live(endpoint_id),
        true,
    );
    let effects = state.drive_delivery_scan("pty_send", fact)?;
    for effect in effects {
        if let crate::reconcile::DeliveryEffect::Inject {
            endpoint_id,
            event_ids,
            ..
        } = effect
        {
            let injected =
                inject_planned_messages_pty(state, rec, &endpoint_id, &event_ids).await?;
            if injected {
                record_message_injection(&rec.pubkey);
            }
            return Ok(injected);
        }
    }
    Ok(false)
}

fn delivery_scan_fact(
    rec: &crate::state::Session,
    pending_event_ids: Vec<String>,
    endpoint_id: Option<String>,
    endpoint_live: bool,
    force: bool,
) -> crate::reconcile::DeliveryScanFact {
    crate::reconcile::DeliveryScanFact {
        pubkey: rec.pubkey.clone(),
        pending_event_ids,
        endpoint_id,
        endpoint_live,
        last_injected_at: last_message_injection(&rec.pubkey),
        debounce_secs: MESSAGE_INJECT_DEBOUNCE_SECS,
        force,
        at: now_secs(),
    }
}

/// Scans for sessions with unread inbox rows that have a live PTY endpoint,
/// and have not been injected recently.
pub fn ring_doorbells(state: Arc<DaemonState>) {
    tokio::spawn(async move {
        if let Err(e) = ring_doorbells_inner(&state).await {
            tracing::error!(error = %format!("{e:#}"), "ring_doorbells: doorbell scan failed");
        }
    });
}

async fn ring_doorbells_inner(state: &Arc<DaemonState>) -> Result<()> {
    let sessions: Vec<crate::state::Session> = state.with_store(|s| {
        let alive = match s.list_alive_sessions() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "ring_doorbells: list_alive_sessions failed — skipping doorbell scan this tick");
                return Vec::new();
            }
        };
        let active_ids: HashSet<String> = alive.iter().map(|rec| rec.pubkey.clone()).collect();
        prune_debounce(&active_ids);
        alive
    });

    for rec in sessions {
        let sid = rec.pubkey.clone();
        let pending = match state.with_store(|s| s.peek_pending_for_pubkey(&rec.pubkey)) {
            Ok(pending) => pending,
            Err(e) => {
                tracing::error!(pubkey = %sid, error = %e, "ring_doorbells: peek_pending_for_pubkey failed");
                state.emit_delivery_failure(
                    &rec.channel_h,
                    &rec.agent_slug,
                    &sid,
                    format!("failed to read pending inbox for doorbell scan: {e:#}"),
                );
                continue;
            }
        };
        if pending.is_empty() {
            continue;
        }

        let kind = transport_kind_for_slug(&rec.agent_slug);
        let endpoint_id = match state.with_store(|s| endpoint_id_for(s, &sid, kind)) {
            Ok(endpoint_id) => endpoint_id,
            Err(e) => {
                tracing::error!(pubkey = %sid, error = %e, "ring_doorbells: locator lookup failed — cannot resolve endpoint this tick");
                state.emit_delivery_failure(
                    &rec.channel_h,
                    &rec.agent_slug,
                    &sid,
                    format!("failed to resolve delivery endpoint: {e:#}"),
                );
                continue;
            }
        };

        // The transport that hosts this session decides how its endpoint is
        // probed for liveness AND how a mention is delivered (PTY bracketed paste
        // vs. ACP JSON-RPC). The reconciler receives transport-neutral endpoint
        // id and liveness inputs.
        let endpoint_live = endpoint_id
            .as_deref()
            .is_some_and(|id| endpoint_is_live(kind, id));
        let fact = delivery_scan_fact(
            &rec,
            pending.into_iter().map(|row| row.event_id).collect(),
            endpoint_id,
            endpoint_live,
            false,
        );
        let effects = match state.drive_delivery_scan("doorbell", fact) {
            Ok(effects) => effects,
            Err(e) => {
                state.emit_delivery_failure(
                    &rec.channel_h,
                    &rec.agent_slug,
                    &sid,
                    format!("delivery reconciler failed: {e:#}"),
                );
                continue;
            }
        };
        apply_delivery_effects(state, &rec, kind, effects).await;
    }
    Ok(())
}

async fn apply_delivery_effects(
    state: &Arc<DaemonState>,
    rec: &crate::state::Session,
    kind: TransportKind,
    effects: Vec<crate::reconcile::DeliveryEffect>,
) {
    for effect in effects {
        match effect {
            crate::reconcile::DeliveryEffect::Inject {
                pubkey,
                endpoint_id: endpoint_id,
                event_ids,
            } => {
                // Route the rendered mention to the transport hosting this
                // endpoint: ACP endpoints get a JSON-RPC `deliver`, PTY endpoints
                // the bracketed-paste inject. The reconciler's `endpoint_id` is the
                // transport-neutral endpoint id.
                let result = match kind {
                    TransportKind::Pty => {
                        inject_planned_messages_pty(state, rec, &endpoint_id, &event_ids).await
                    }
                    TransportKind::Acp => {
                        inject_planned_messages_acp(state, rec, &endpoint_id, &event_ids).await
                    }
                };
                match result {
                    Ok(true) => {
                        record_message_injection(&pubkey);
                        if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                            eprintln!(
                                "[{}] pending messages delivered to endpoint {endpoint_id} for {pubkey}",
                                kind.as_str()
                            );
                        }
                    }
                    Ok(false) => {}
                    Err(e) => state.emit_delivery_failure(
                        &rec.channel_h,
                        &rec.agent_slug,
                        &pubkey,
                        format!(
                            "pending message delivery failed for {} endpoint {endpoint_id}: {e:#}",
                            kind.as_str()
                        ),
                    ),
                }
            }
            crate::reconcile::DeliveryEffect::RetryAfter { pubkey, delay_secs } => {
                schedule_delivery_retry(state.clone(), pubkey, delay_secs)
            }
            crate::reconcile::DeliveryEffect::ClearDeadEndpoint { pubkey } => {
                // Delivery owns the transport locator only. Runtime lifecycle is
                // the sole owner of generation-fenced session teardown.
                let _ = state.with_store(|s| s.clear_locator_kind(&pubkey, locator_kind(kind)));
            }
        }
    }
}

fn schedule_delivery_retry(state: Arc<DaemonState>, pubkey: String, delay_secs: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(delay_secs.max(1))).await;
        if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
            eprintln!("[pty] retrying deferred delivery for {pubkey}");
        }
        ring_doorbells(state);
    });
}
