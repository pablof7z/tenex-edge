use std::collections::BTreeMap;

use trellis_core::{
    AuditExplanationLevel, DependencyList, GraphResult, InputNode, MapDiff, PlanContext, PlanError,
    ResourceKey, ResourcePlan, Transaction, TransactionOptions,
};

use crate::reconcile::labels::NodeLabels;

use super::{DeliveryAction, DeliveryCommand, DeliveryScanFact};

#[derive(Clone, Copy)]
pub(crate) struct SessionNodes {
    pending: InputNode<Vec<String>>,
    endpoint_id: InputNode<Option<String>>,
    endpoint_live: InputNode<bool>,
    last_injected_at: InputNode<u64>,
    debounce_secs: InputNode<u64>,
    now: InputNode<u64>,
    force: InputNode<bool>,
    seq: InputNode<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decision {
    command: Option<DeliveryCommand>,
    seq: u64,
}

pub(crate) fn delivery_key(id: &str) -> ResourceKey {
    ResourceKey::from_segments(["delivery", id])
}

pub(crate) fn opts() -> TransactionOptions {
    TransactionOptions::default().with_audit_explanations(AuditExplanationLevel::DependencyPaths)
}

pub(crate) fn ensure_session(
    tx: &mut Transaction<'_, DeliveryCommand>,
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

pub(crate) fn stage_scan(
    tx: &mut Transaction<'_, DeliveryCommand>,
    nodes: &SessionNodes,
    fact: &DeliveryScanFact,
    seq: u64,
) -> GraphResult<()> {
    tx.set_input(nodes.pending, fact.pending_event_ids.clone())?;
    tx.set_input(nodes.endpoint_id, fact.endpoint_id.clone())?;
    tx.set_input(nodes.endpoint_live, fact.endpoint_live)?;
    tx.set_input(nodes.last_injected_at, fact.last_injected_at.unwrap_or(0))?;
    tx.set_input(nodes.debounce_secs, fact.debounce_secs)?;
    tx.set_input(nodes.now, fact.at)?;
    tx.set_input(nodes.force, fact.force)?;
    tx.set_input(nodes.seq, seq)
}

fn stage_session(
    tx: &mut Transaction<'_, DeliveryCommand>,
    labels: &mut NodeLabels,
    pubkey: &str,
) -> GraphResult<SessionNodes> {
    let scope = tx.create_scope(format!("delivery-{pubkey}"))?;
    let pending = input(
        tx,
        labels,
        pubkey,
        "pending_event_ids",
        Vec::<String>::new(),
    )?;
    let endpoint_id = input(tx, labels, pubkey, "endpoint_id", None::<String>)?;
    let endpoint_live = input(tx, labels, pubkey, "endpoint_live", false)?;
    let last = input(tx, labels, pubkey, "last_injected_at", 0u64)?;
    let debounce = input(tx, labels, pubkey, "debounce_secs", 0u64)?;
    let now = input(tx, labels, pubkey, "now", 0u64)?;
    let force = input(tx, labels, pubkey, "force", false)?;
    let seq = input(tx, labels, pubkey, "request_seq", 0u64)?;
    let nodes = SessionNodes {
        pending,
        endpoint_id,
        endpoint_live,
        last_injected_at: last,
        debounce_secs: debounce,
        now,
        force,
        seq,
    };
    let decision = decision_node(tx, labels, pubkey, nodes)?;
    let id = pubkey.to_string();
    let coll = tx.map_collection::<String, Decision>(
        format!("delivery-{pubkey}-coll"),
        DependencyList::new([decision.id()])?,
        move |ctx| {
            let current = ctx.derived(decision)?.clone();
            if current.command.is_some() {
                Ok(BTreeMap::from([(id.clone(), current)]))
            } else {
                Ok(BTreeMap::new())
            }
        },
    )?;
    labels.record(coll.id(), format!("delivery/{pubkey}/coll"));
    tx.map_resource_planner(coll, scope, plan_delivery)?;
    Ok(nodes)
}

fn input<T: Clone + PartialEq + Send + Sync + 'static>(
    tx: &mut Transaction<'_, DeliveryCommand>,
    labels: &mut NodeLabels,
    pubkey: &str,
    name: &str,
    value: T,
) -> GraphResult<InputNode<T>> {
    let node = tx.input::<T>(format!("delivery-{pubkey}-{name}"))?;
    labels.record(node.id(), format!("delivery/{pubkey}/{name}"));
    tx.set_input(node, value)?;
    Ok(node)
}

fn decision_node(
    tx: &mut Transaction<'_, DeliveryCommand>,
    labels: &mut NodeLabels,
    id: &str,
    nodes: SessionNodes,
) -> GraphResult<trellis_core::DerivedNode<Decision>> {
    let pubkey = id.to_string();
    let decision = tx.derived(
        format!("delivery-{id}-decision"),
        DependencyList::new([
            nodes.pending.id(),
            nodes.endpoint_id.id(),
            nodes.endpoint_live.id(),
            nodes.last_injected_at.id(),
            nodes.debounce_secs.id(),
            nodes.now.id(),
            nodes.force.id(),
            nodes.seq.id(),
        ])?,
        move |ctx| {
            let command = decide(
                &pubkey,
                ctx.input(nodes.pending)?.clone(),
                ctx.input(nodes.endpoint_id)?.clone(),
                *ctx.input(nodes.endpoint_live)?,
                *ctx.input(nodes.last_injected_at)?,
                *ctx.input(nodes.debounce_secs)?,
                *ctx.input(nodes.now)?,
                *ctx.input(nodes.force)?,
            );
            Ok(Decision {
                command,
                seq: *ctx.input(nodes.seq)?,
            })
        },
    )?;
    labels.record(decision.id(), format!("delivery/{id}/decision"));
    Ok(decision)
}

#[allow(clippy::too_many_arguments)]
fn decide(
    pubkey: &str,
    event_ids: Vec<String>,
    endpoint_id: Option<String>,
    endpoint_live: bool,
    last_injected_at: u64,
    debounce_secs: u64,
    now: u64,
    force: bool,
) -> Option<DeliveryCommand> {
    if event_ids.is_empty() {
        return None;
    }
    let Some(endpoint_id) = endpoint_id else {
        return Some(command(
            pubkey,
            DeliveryAction::DeferNoEndpoint,
            event_ids,
            None,
            None,
        ));
    };
    if !endpoint_live {
        return Some(command(
            pubkey,
            DeliveryAction::ClearDeadEndpoint,
            event_ids,
            Some(endpoint_id),
            None,
        ));
    }
    let elapsed = now.saturating_sub(last_injected_at);
    if !force && last_injected_at > 0 && elapsed < debounce_secs {
        let retry_after = debounce_secs.saturating_sub(elapsed).max(1);
        return Some(command(
            pubkey,
            DeliveryAction::DeferDebounced,
            event_ids,
            Some(endpoint_id),
            Some(retry_after),
        ));
    }
    Some(command(
        pubkey,
        DeliveryAction::Inject,
        event_ids,
        Some(endpoint_id),
        None,
    ))
}

fn command(
    pubkey: &str,
    action: DeliveryAction,
    event_ids: Vec<String>,
    endpoint_id: Option<String>,
    retry_after_secs: Option<u64>,
) -> DeliveryCommand {
    DeliveryCommand {
        pubkey: pubkey.to_string(),
        action,
        event_ids,
        endpoint_id,
        retry_after_secs,
    }
}

fn plan_delivery(
    ctx: &PlanContext<MapDiff<String, Decision>>,
) -> Result<ResourcePlan<DeliveryCommand>, PlanError> {
    let mut plan = ResourcePlan::new();
    for added in &ctx.diff().added {
        let (id, decision) = &added.value;
        if let Some(command) = command_of(decision) {
            plan.open(delivery_key(id), ctx.scope(), command);
        }
    }
    for updated in &ctx.diff().updated {
        if let Some(command) = command_of(&updated.current) {
            plan.replace(delivery_key(&updated.key), ctx.scope(), command);
        }
    }
    for removed in &ctx.diff().removed {
        plan.close(delivery_key(&removed.value.0), ctx.scope());
    }
    Ok(plan)
}

fn command_of(decision: &Decision) -> Option<DeliveryCommand> {
    let _ = decision.seq;
    decision.command.clone()
}
