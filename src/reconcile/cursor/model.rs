use std::collections::BTreeMap;

use trellis_core::{
    AuditExplanationLevel, DependencyList, GraphResult, InputNode, MapDiff, PlanContext, PlanError,
    ResourceKey, ResourcePlan, Transaction, TransactionOptions,
};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::NodeLabels;

use super::{CursorCommand, CursorFrame, CursorSeed};

#[derive(Clone, Copy)]
pub(crate) struct SessionNodes {
    pub(super) current: InputNode<u64>,
    pub(super) observed: InputNode<u64>,
    pub(super) at: InputNode<u64>,
    pub(super) working: InputNode<bool>,
    pub(super) seq: InputNode<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decision {
    cursor_before: u64,
    cursor_after: u64,
    delta_since: Option<u64>,
    frame: CursorFrame,
    seq: u64,
}

pub(crate) fn fact_seed(fact: &InputFact) -> Option<(String, u64)> {
    match fact {
        InputFact::TurnCheckRequested {
            session_id,
            observed_cursor,
            ..
        } => Some((session_id.clone(), *observed_cursor)),
        _ => None,
    }
}

pub(crate) fn cursor_key(id: &str) -> ResourceKey {
    ResourceKey::from_segments(["cursor", id])
}

pub(crate) fn opts() -> TransactionOptions {
    TransactionOptions::default().with_audit_explanations(AuditExplanationLevel::DependencyPaths)
}

pub(crate) fn ensure_session(
    tx: &mut Transaction<'_, CursorCommand>,
    labels: &mut NodeLabels,
    sessions: &mut BTreeMap<String, SessionNodes>,
    seed: &CursorSeed,
) -> GraphResult<SessionNodes> {
    if let Some(nodes) = sessions.get(&seed.session_id).copied() {
        return Ok(nodes);
    }
    let nodes = stage_session(tx, labels, seed)?;
    sessions.insert(seed.session_id.clone(), nodes);
    Ok(nodes)
}

pub(crate) fn stage_fact(
    tx: &mut Transaction<'_, CursorCommand>,
    nodes: &SessionNodes,
    current_cursor: u64,
    fact: &InputFact,
    seq: u64,
) -> GraphResult<()> {
    if let InputFact::TurnCheckRequested {
        observed_cursor,
        working,
        at,
        ..
    } = fact
    {
        tx.set_input(nodes.current, current_cursor)?;
        tx.set_input(nodes.observed, *observed_cursor)?;
        tx.set_input(nodes.working, *working)?;
        tx.set_input(nodes.at, *at)?;
        tx.set_input(nodes.seq, seq)?;
    }
    Ok(())
}

fn stage_session(
    tx: &mut Transaction<'_, CursorCommand>,
    labels: &mut NodeLabels,
    seed: &CursorSeed,
) -> GraphResult<SessionNodes> {
    let id = seed.session_id.clone();
    let scope = tx.create_scope(format!("cursor-{id}"))?;
    let current = tx.input::<u64>(format!("cursor-{id}-current"))?;
    labels.record(current.id(), format!("cursor/{id}/current_cursor"));
    tx.set_input(current, seed.seen_cursor)?;
    let observed = tx.input::<u64>(format!("cursor-{id}-observed"))?;
    labels.record(observed.id(), format!("cursor/{id}/observed_cursor"));
    tx.set_input(observed, seed.seen_cursor)?;
    let at = tx.input::<u64>(format!("cursor-{id}-now"))?;
    labels.record(at.id(), format!("cursor/{id}/now"));
    tx.set_input(at, seed.seen_cursor)?;
    let working = tx.input::<bool>(format!("cursor-{id}-working"))?;
    labels.record(working.id(), format!("cursor/{id}/working"));
    tx.set_input(working, false)?;
    let seq = tx.input::<u64>(format!("cursor-{id}-seq"))?;
    labels.record(seq.id(), format!("cursor/{id}/request_seq"));
    tx.set_input(seq, 0)?;
    let nodes = SessionNodes {
        current,
        observed,
        at,
        working,
        seq,
    };
    let decision = decision_node(tx, labels, &id, nodes)?;
    let coll = tx.map_collection::<String, Decision>(
        format!("cursor-{id}-coll"),
        DependencyList::new([decision.id()])?,
        move |ctx| {
            Ok(BTreeMap::from([(
                id.clone(),
                ctx.derived(decision)?.clone(),
            )]))
        },
    )?;
    labels.record(coll.id(), format!("cursor/{}/coll", seed.session_id));
    tx.map_resource_planner(coll, scope, plan_cursor)?;
    Ok(nodes)
}

fn decision_node(
    tx: &mut Transaction<'_, CursorCommand>,
    labels: &mut NodeLabels,
    id: &str,
    nodes: SessionNodes,
) -> GraphResult<trellis_core::DerivedNode<Decision>> {
    let decision = tx.derived(
        format!("cursor-{id}-decision"),
        DependencyList::new([
            nodes.current.id(),
            nodes.observed.id(),
            nodes.at.id(),
            nodes.working.id(),
            nodes.seq.id(),
        ])?,
        move |ctx| {
            let current = *ctx.input(nodes.current)?;
            let observed = *ctx.input(nodes.observed)?;
            let at = *ctx.input(nodes.at)?;
            let working = *ctx.input(nodes.working)?;
            let frame = working && observed == current && at > current;
            Ok(Decision {
                cursor_before: current,
                cursor_after: if frame { at } else { current },
                delta_since: frame.then_some(current),
                frame: if frame {
                    CursorFrame::HookFrame
                } else {
                    CursorFrame::NoFrame
                },
                seq: *ctx.input(nodes.seq)?,
            })
        },
    )?;
    labels.record(decision.id(), format!("cursor/{id}/decision"));
    Ok(decision)
}

fn plan_cursor(
    ctx: &PlanContext<MapDiff<String, Decision>>,
) -> Result<ResourcePlan<CursorCommand>, PlanError> {
    let mut plan = ResourcePlan::new();
    for added in &ctx.diff().added {
        let (id, decision) = &added.value;
        plan.open(cursor_key(id), ctx.scope(), command_of(id, decision));
    }
    for updated in &ctx.diff().updated {
        plan.replace(
            cursor_key(&updated.key),
            ctx.scope(),
            command_of(&updated.key, &updated.current),
        );
    }
    Ok(plan)
}

fn command_of(id: &str, decision: &Decision) -> CursorCommand {
    let _ = decision.seq;
    CursorCommand {
        session_id: id.to_string(),
        cursor_before: decision.cursor_before,
        cursor_after: decision.cursor_after,
        delta_since: decision.delta_since,
        frame: decision.frame.clone(),
    }
}
