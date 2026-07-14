use std::collections::BTreeMap;

use trellis_core::{
    AuditExplanationLevel, DependencyList, GraphResult, InputNode, MapDiff, PlanContext, PlanError,
    ResourceKey, ResourcePlan, Transaction, TransactionOptions,
};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::NodeLabels;

use super::{TurnCommand, TurnProjectionSeed};

#[derive(Clone, Copy)]
pub(crate) struct SessionNodes {
    pub(super) started_at: InputNode<u64>,
    pub(super) ended_at: InputNode<u64>,
    pub(super) transcript_ref: InputNode<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Projection {
    working: bool,
    turn_started_at: u64,
    transcript_ref: Option<String>,
}

pub(crate) fn turn_key(id: &str) -> ResourceKey {
    ResourceKey::from_segments(["turn_lifecycle", id])
}

pub(crate) fn opts() -> TransactionOptions {
    TransactionOptions::default().with_audit_explanations(AuditExplanationLevel::DependencyPaths)
}

pub(crate) fn fact_pubkey(fact: &InputFact) -> Option<&str> {
    match fact {
        InputFact::TurnStarted { pubkey, .. }
        | InputFact::TurnEnded { pubkey, .. }
        | InputFact::TranscriptWindowCaptured { pubkey, .. } => Some(pubkey),
        _ => None,
    }
}

pub(crate) fn ensure_session(
    tx: &mut Transaction<'_, TurnCommand>,
    labels: &mut NodeLabels,
    sessions: &mut BTreeMap<String, SessionNodes>,
    seed: &TurnProjectionSeed,
) -> GraphResult<SessionNodes> {
    if let Some(nodes) = sessions.get(&seed.pubkey).copied() {
        return Ok(nodes);
    }
    let nodes = stage_session(tx, labels, seed)?;
    sessions.insert(seed.pubkey.clone(), nodes);
    Ok(nodes)
}

pub(crate) fn stage_fact(
    sessions: &BTreeMap<String, SessionNodes>,
    fact: &InputFact,
    tx: &mut Transaction<'_, TurnCommand>,
) -> GraphResult<()> {
    match fact {
        InputFact::TurnStarted { pubkey, at } => {
            if let Some(nodes) = sessions.get(pubkey) {
                tx.set_input(nodes.started_at, *at)?;
            }
        }
        InputFact::TurnEnded { pubkey, at } => {
            if let Some(nodes) = sessions.get(pubkey) {
                tx.set_input(nodes.ended_at, *at)?;
            }
        }
        InputFact::TranscriptWindowCaptured {
            pubkey,
            window_hash,
            ..
        } => {
            if let Some(nodes) = sessions.get(pubkey) {
                tx.set_input(nodes.transcript_ref, Some(window_hash.clone()))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn stage_session(
    tx: &mut Transaction<'_, TurnCommand>,
    labels: &mut NodeLabels,
    seed: &TurnProjectionSeed,
) -> GraphResult<SessionNodes> {
    let id = seed.pubkey.clone();
    let scope = tx.create_scope(format!("turn-lifecycle-{id}"))?;
    let started = tx.input::<u64>(format!("turn-{id}-started-at"))?;
    labels.record(started.id(), format!("turn_lifecycle/{id}/turn_started"));
    tx.set_input(
        started,
        if seed.working {
            seed.turn_started_at
        } else {
            0
        },
    )?;
    let ended = tx.input::<u64>(format!("turn-{id}-ended-at"))?;
    labels.record(ended.id(), format!("turn_lifecycle/{id}/turn_ended"));
    tx.set_input(ended, 0)?;
    let transcript = tx.input::<Option<String>>(format!("turn-{id}-transcript"))?;
    labels.record(
        transcript.id(),
        format!("turn_lifecycle/{id}/transcript_window"),
    );
    tx.set_input(transcript, seed.transcript_ref.clone())?;

    let projection = tx.derived(
        format!("turn-{id}-projection"),
        DependencyList::new([started.id(), ended.id(), transcript.id()])?,
        move |ctx| {
            let started_at = *ctx.input(started)?;
            let ended_at = *ctx.input(ended)?;
            let working = started_at > 0 && started_at > ended_at;
            Ok(Projection {
                working,
                turn_started_at: if working { started_at } else { 0 },
                transcript_ref: ctx.input(transcript)?.clone(),
            })
        },
    )?;
    labels.record(projection.id(), format!("turn_lifecycle/{id}/projection"));
    let coll = tx.map_collection::<String, Projection>(
        format!("turn-{id}-coll"),
        DependencyList::new([projection.id()])?,
        move |ctx| {
            Ok(BTreeMap::from([(
                id.clone(),
                ctx.derived(projection)?.clone(),
            )]))
        },
    )?;
    labels.record(coll.id(), format!("turn_lifecycle/{}/coll", seed.pubkey));
    tx.map_resource_planner(coll, scope, plan_projection)?;
    Ok(SessionNodes {
        started_at: started,
        ended_at: ended,
        transcript_ref: transcript,
    })
}

fn plan_projection(
    ctx: &PlanContext<MapDiff<String, Projection>>,
) -> Result<ResourcePlan<TurnCommand>, PlanError> {
    let mut plan = ResourcePlan::new();
    for added in &ctx.diff().added {
        let (id, projection) = &added.value;
        plan.open(turn_key(id), ctx.scope(), command_of(id, projection));
    }
    for updated in &ctx.diff().updated {
        plan.replace(
            turn_key(&updated.key),
            ctx.scope(),
            command_of(&updated.key, &updated.current),
        );
    }
    for removed in &ctx.diff().removed {
        plan.close(turn_key(&removed.value.0), ctx.scope());
    }
    Ok(plan)
}

fn command_of(id: &str, projection: &Projection) -> TurnCommand {
    TurnCommand {
        pubkey: id.to_string(),
        working: projection.working,
        turn_started_at: projection.turn_started_at,
        transcript_ref: projection.transcript_ref.clone(),
    }
}
