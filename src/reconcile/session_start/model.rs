use std::collections::BTreeMap;

use trellis_core::{
    AuditExplanationLevel, DependencyList, GraphResult, InputNode, MapDiff, PlanContext, PlanError,
    ResourceKey, ResourcePlan, Transaction, TransactionOptions,
};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::NodeLabels;
use crate::reconcile::{SessionStartFailedFact, SessionStartRequestFact};

use super::{plan_from_request, SessionStartAction, SessionStartCommand};

const OUTCOME_PENDING: i64 = 0;
const OUTCOME_STARTED: i64 = 1;
const OUTCOME_FAILED: i64 = 2;

#[derive(Clone, Copy)]
pub(crate) struct SessionNodes {
    pub(super) request: InputNode<Option<SessionStartRequestFact>>,
    pub(super) outcome: InputNode<i64>,
    pub(super) failure_stage: InputNode<String>,
    pub(super) failure_error: InputNode<String>,
    pub(super) seq: InputNode<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decision {
    command: SessionStartCommand,
    seq: u64,
}

pub(crate) fn fact_pubkey(fact: &InputFact) -> Option<&str> {
    match fact {
        InputFact::SessionStartRequested(req) => Some(req.pubkey.as_str()),
        InputFact::SessionStarted { pubkey, .. } => Some(pubkey.as_str()),
        InputFact::SessionStartFailed(SessionStartFailedFact { pubkey, .. }) => {
            Some(pubkey.as_str())
        }
        _ => None,
    }
}

pub(crate) fn session_key(pubkey: &str) -> ResourceKey {
    ResourceKey::from_segments(["session_start", pubkey])
}

pub(crate) fn opts() -> TransactionOptions {
    TransactionOptions::default().with_audit_explanations(AuditExplanationLevel::DependencyPaths)
}

pub(crate) fn ensure_session(
    tx: &mut Transaction<'_, SessionStartCommand>,
    labels: &mut NodeLabels,
    sessions: &mut BTreeMap<String, SessionNodes>,
    pubkey: &str,
) -> GraphResult<SessionNodes> {
    if let Some(nodes) = sessions.get(pubkey).copied() {
        return Ok(nodes);
    }
    let nodes = stage_session(tx, labels, pubkey)?;
    sessions.insert(pubkey.to_string(), nodes);
    Ok(nodes)
}

pub(crate) fn stage_fact(
    tx: &mut Transaction<'_, SessionStartCommand>,
    nodes: &SessionNodes,
    fact: &InputFact,
    seq: u64,
) -> GraphResult<()> {
    match fact {
        InputFact::SessionStartRequested(req) => {
            tx.set_input(nodes.request, Some(req.clone()))?;
            tx.set_input(nodes.outcome, OUTCOME_PENDING)?;
            tx.set_input(nodes.failure_stage, String::new())?;
            tx.set_input(nodes.failure_error, String::new())?;
            tx.set_input(nodes.seq, seq)?;
        }
        InputFact::SessionStarted { .. } => {
            tx.set_input(nodes.outcome, OUTCOME_STARTED)?;
            tx.set_input(nodes.failure_stage, String::new())?;
            tx.set_input(nodes.failure_error, String::new())?;
            tx.set_input(nodes.seq, seq)?;
        }
        InputFact::SessionStartFailed(failed) => {
            tx.set_input(nodes.outcome, OUTCOME_FAILED)?;
            tx.set_input(nodes.failure_stage, failed.stage.clone())?;
            tx.set_input(nodes.failure_error, failed.error.clone())?;
            tx.set_input(nodes.seq, seq)?;
        }
        _ => {}
    }
    Ok(())
}

fn stage_session(
    tx: &mut Transaction<'_, SessionStartCommand>,
    labels: &mut NodeLabels,
    pubkey: &str,
) -> GraphResult<SessionNodes> {
    let scope = tx.create_scope(format!("session-start-{pubkey}"))?;
    let request = input(tx, labels, pubkey, "request", None)?;
    let outcome = input(tx, labels, pubkey, "outcome", OUTCOME_PENDING)?;
    let failure_stage = input(tx, labels, pubkey, "failure_stage", String::new())?;
    let failure_error = input(tx, labels, pubkey, "failure_error", String::new())?;
    let seq = input(tx, labels, pubkey, "seq", 0u64)?;
    let nodes = SessionNodes {
        request,
        outcome,
        failure_stage,
        failure_error,
        seq,
    };
    let decision = tx.derived(
        format!("session-start-{pubkey}-decision"),
        DependencyList::new([
            request.id(),
            outcome.id(),
            failure_stage.id(),
            failure_error.id(),
            seq.id(),
        ])?,
        move |ctx| {
            let Some(req) = ctx.input(request)?.clone() else {
                return Ok(None);
            };
            let outcome = *ctx.input(outcome)?;
            let command = command_from_inputs(
                &req,
                outcome,
                ctx.input(failure_stage)?.clone(),
                ctx.input(failure_error)?.clone(),
            );
            Ok(Some(Decision {
                command,
                seq: *ctx.input(seq)?,
            }))
        },
    )?;
    labels.record(decision.id(), format!("session_start/{pubkey}/decision"));
    let id = pubkey.to_string();
    let coll = tx.map_collection::<String, Decision>(
        format!("session-start-{pubkey}-coll"),
        DependencyList::new([decision.id()])?,
        move |ctx| {
            Ok(ctx
                .derived(decision)?
                .clone()
                .map(|d| BTreeMap::from([(id.clone(), d)]))
                .unwrap_or_default())
        },
    )?;
    labels.record(coll.id(), format!("session_start/{pubkey}/coll"));
    tx.map_resource_planner(coll, scope, plan_session_start)?;
    Ok(nodes)
}

fn input<T: Clone + PartialEq + Send + Sync + 'static>(
    tx: &mut Transaction<'_, SessionStartCommand>,
    labels: &mut NodeLabels,
    pubkey: &str,
    name: &str,
    value: T,
) -> GraphResult<InputNode<T>> {
    let node = tx.input::<T>(format!("session-start-{pubkey}-{name}"))?;
    labels.record(node.id(), format!("session_start/{pubkey}/{name}"));
    tx.set_input(node, value)?;
    Ok(node)
}

fn command_from_inputs(
    req: &SessionStartRequestFact,
    outcome: i64,
    failure_stage: String,
    failure_error: String,
) -> SessionStartCommand {
    let action = match outcome {
        OUTCOME_STARTED => SessionStartAction::RecordStarted,
        OUTCOME_FAILED => SessionStartAction::RecordFailed,
        _ if req.already_running => SessionStartAction::Reassert,
        _ => SessionStartAction::Execute,
    };
    SessionStartCommand {
        pubkey: req.pubkey.clone(),
        action,
        plan: plan_from_request(req),
        failure_stage: (!failure_stage.is_empty()).then_some(failure_stage),
        failure_error: (!failure_error.is_empty()).then_some(failure_error),
    }
}

fn plan_session_start(
    ctx: &PlanContext<MapDiff<String, Decision>>,
) -> Result<ResourcePlan<SessionStartCommand>, PlanError> {
    let mut plan = ResourcePlan::new();
    for added in &ctx.diff().added {
        let (id, decision) = &added.value;
        plan.open(session_key(id), ctx.scope(), command_of(decision));
    }
    for updated in &ctx.diff().updated {
        plan.replace(
            session_key(&updated.key),
            ctx.scope(),
            command_of(&updated.current),
        );
    }
    Ok(plan)
}

fn command_of(decision: &Decision) -> SessionStartCommand {
    let _ = decision.seq;
    decision.command.clone()
}
