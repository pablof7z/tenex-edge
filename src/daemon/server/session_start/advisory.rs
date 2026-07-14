use super::*;
use crate::reconcile::session_start::{plan_from_request, SessionStartAction};
use crate::reconcile::{
    InputFact, SessionStartCommand, SessionStartFailedFact, SessionStartRequestFact,
};

pub(super) fn observed_command(req: &SessionStartRequestFact) -> SessionStartCommand {
    SessionStartCommand {
        pubkey: req.pubkey.clone(),
        action: if req.already_running {
            SessionStartAction::Reassert
        } else {
            SessionStartAction::Execute
        },
        plan: plan_from_request(req),
        failure_stage: None,
        failure_error: None,
    }
}

pub(super) fn drive_request(
    state: &Arc<DaemonState>,
    req: SessionStartRequestFact,
) -> Result<SessionStartCommand> {
    let observed = observed_command(&req);
    let command = drive_fact(
        state,
        "session_start_requested",
        InputFact::SessionStartRequested(req.clone()),
    )?
    .ok_or_else(|| anyhow::anyhow!("session_start advisory emitted no staged intent"))?;
    if command != observed {
        tracing::warn!(pubkey = %req.pubkey, ?observed, derived = ?command,
            "session_start advisory shadow comparison mismatch");
    }
    Ok(command)
}

pub(super) fn record_started(
    state: &Arc<DaemonState>,
    pubkey: &str,
    channel_h: &str,
    pid: Option<i32>,
    at: u64,
) {
    if let Err(error) = drive_fact(
        state,
        "session_started",
        InputFact::SessionStarted {
            pubkey: pubkey.to_string(),
            channel_h: Some(channel_h.to_string()),
            pid,
            at,
        },
    ) {
        tracing::warn!(pubkey, %error, "failed to record session_start success fact");
    }
}

pub(super) fn record_failed(
    state: &Arc<DaemonState>,
    pubkey: &str,
    stage: &str,
    error: &anyhow::Error,
    at: u64,
) {
    if let Err(record_error) = drive_fact(
        state,
        "session_start_failed",
        InputFact::SessionStartFailed(SessionStartFailedFact {
            pubkey: pubkey.to_string(),
            stage: stage.to_string(),
            error: format!("{error:#}"),
            at,
        }),
    ) {
        tracing::warn!(pubkey, stage, %record_error,
            "failed to record session_start failure fact");
    }
}

fn drive_fact(
    state: &Arc<DaemonState>,
    trigger: &str,
    fact: InputFact,
) -> Result<Option<SessionStartCommand>> {
    let start = std::time::Instant::now();
    let facts = vec![fact.clone()];
    let trigger_ref = fact_pubkey(&fact).map(str::to_string);
    let (preview, outcome, commit) = {
        let mut reconciler = state
            .session_start
            .lock()
            .expect("session_start mutex poisoned");
        let preview = reconciler
            .preview_fact(&fact)
            .map_err(|error| anyhow::anyhow!("session_start preview failed: {error:?}"))?
            .ok_or_else(|| anyhow::anyhow!("unsupported session_start fact"))?;
        let outcome = reconciler
            .drive(fact)
            .map_err(|error| anyhow::anyhow!("session_start drive failed: {error:?}"))?;
        let mut commit = crate::reconcile::CommitFacts::from_result(
            reconciler.labels(),
            &outcome.result,
            reconciler.graph_node_count(),
        );
        commit.graph_resources = reconciler.state_rows().len() as i64;
        (preview.result, outcome, commit)
    };
    if !crate::reconcile::preview::command_plans_match(
        preview.resource_plan.commands(),
        outcome.result.resource_plan.commands(),
    ) {
        anyhow::bail!("session_start advisory blocked: committed plan was not previewed first");
    }
    let created_at = crate::instrument::now_millis();
    state.with_store(|store| {
        crate::instrument::record_commit(
            store,
            "session_start",
            trigger,
            trigger_ref.as_deref(),
            &commit,
            start.elapsed().as_micros() as i64,
            created_at,
        );
        crate::replay_capsules::record_many(
            store,
            "session_start",
            trigger,
            trigger_ref.as_deref(),
            facts,
            created_at,
        );
    });
    Ok(outcome.command)
}

fn fact_pubkey(fact: &InputFact) -> Option<&str> {
    match fact {
        InputFact::SessionStartRequested(request) => Some(&request.pubkey),
        InputFact::SessionStarted { pubkey, .. } => Some(pubkey),
        InputFact::SessionStartFailed(failed) => Some(&failed.pubkey),
        _ => None,
    }
}
