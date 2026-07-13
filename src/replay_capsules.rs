//! Replay-capsule instrumentation: capture versioned Trellis input scripts at
//! reconciler drive seams without blocking the hot path.

use crate::reconcile::InputFact;
use crate::state::trellis_replay_capsules::NewReplayCapsule;
use crate::state::Store;
use trellis_testing::{DataTransactionScript, TRACE_FORMAT_VERSION};

macro_rules! status_fact {
    (started, $p:expr, $aref:expr, $session:expr, $channels:expr, $automatic_delivery:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::SessionStarted(
            $crate::reconcile::StatusSessionStartedArgs {
                pubkey: $aref.pubkey.clone(),
                host: $p.host.clone(),
                slug: $aref.slug.clone(),
                rel_cwd: $p.rel_cwd.clone(),
                dispatch_event: $p.dispatch_event.clone(),
                channels: $channels.clone(),
                working: $session.working,
                automatic_delivery: $automatic_delivery,
                title: $session.title.clone(),
                activity: $session.activity.clone(),
                at: $at,
            },
        ))
    };
    (tick, $pubkey:expr, $automatic_delivery:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::Tick {
            pubkey: $pubkey.clone(),
            automatic_delivery: $automatic_delivery,
            at: $at,
        })
    };
    (distill, $pubkey:expr, $labels:expr, $window_hash:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive(
            $crate::reconcile::StatusDrive::DistillCompleted {
                pubkey: $pubkey.clone(),
                title: $labels.title.clone(),
                activity: $labels.activity.clone(),
                window_hash: $window_hash.clone(),
                at: $at,
            },
        )
    };
    (title, $pubkey:expr, $title:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::TitleSet {
            pubkey: $pubkey.clone(),
            title: $title.clone(),
            at: $at,
        })
    };
    (turn, $pubkey:expr, $working:expr, $at:expr) => {
        if $working {
            $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::TurnStarted {
                pubkey: $pubkey.clone(),
                at: $at,
            })
        } else {
            $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::TurnEnded {
                pubkey: $pubkey.clone(),
                at: $at,
            })
        }
    };
    (channels, $pubkey:expr, $channels:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::ChannelsChanged {
            pubkey: $pubkey.clone(),
            channels: $channels.clone(),
            at: $at,
        })
    };
    (ended, $pubkey:expr, $at:expr) => {
        $crate::reconcile::InputFact::StatusDrive($crate::reconcile::StatusDrive::SessionEnded {
            pubkey: $pubkey.clone(),
            at: $at,
        })
    };
}

pub(crate) use status_fact;

/// Stable content pointer for emitted hook/context text.
pub fn text_hash(text: &str) -> String {
    crate::instrument::window_hash(text)
}

/// Record one replay capsule at a reconciler drive seam. The stored payload is a
/// versioned Trellis data script with exactly one [`InputFact`] operation.
/// Disabled when the replay-capsule gate (or the shared hook-call-log gate) is
/// set to an off value.
pub fn record(
    store: &Store,
    surface: &str,
    trigger_kind: &str,
    trigger_ref: Option<&str>,
    fact: InputFact,
    created_at: i64,
) {
    record_many(
        store,
        surface,
        trigger_kind,
        trigger_ref,
        vec![fact],
        created_at,
    )
}

pub fn record_many(
    store: &Store,
    surface: &str,
    trigger_kind: &str,
    trigger_ref: Option<&str>,
    facts: Vec<InputFact>,
    created_at: i64,
) {
    if !enabled() {
        return;
    }
    let mut script = DataTransactionScript::new();
    for (index, fact) in facts.into_iter().enumerate() {
        script
            .step(capsule_step_name(surface, trigger_kind, trigger_ref, index))
            .operation(fact)
            .commit();
    }
    let script_json = match script.to_json() {
        Ok(json) => json,
        Err(e) => {
            tracing::warn!(surface, error = %e, "replay capsule serialization failed");
            return;
        }
    };
    let row = NewReplayCapsule {
        surface: surface.to_string(),
        trigger_kind: trigger_kind.to_string(),
        trigger_ref: trigger_ref.unwrap_or_default().to_string(),
        script_json,
        format_version: TRACE_FORMAT_VERSION as i64,
        created_at,
    };
    if let Err(e) = store.record_replay_capsule(&row) {
        tracing::warn!(surface, error = %e, "record_replay_capsule failed");
    }
}

pub fn enabled() -> bool {
    std::env::var("TENEX_EDGE_REPLAY_CAPSULES")
        .ok()
        .or_else(|| std::env::var("TENEX_EDGE_HOOK_CALL_LOG").ok())
        .as_deref()
        .map(gate_is_enabled)
        .unwrap_or(true)
}

fn gate_is_enabled(raw: &str) -> bool {
    !matches!(raw.trim(), "" | "0" | "false" | "off" | "none")
}

fn capsule_step_name(
    surface: &str,
    trigger_kind: &str,
    trigger_ref: Option<&str>,
    index: usize,
) -> String {
    match trigger_ref.filter(|s| !s.is_empty()) {
        Some(reference) => format!("{surface}/{trigger_kind}/{reference}/{index}"),
        None => format!("{surface}/{trigger_kind}/{index}"),
    }
}
