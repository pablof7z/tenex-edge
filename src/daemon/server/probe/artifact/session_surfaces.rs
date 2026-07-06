use super::{plan_artifact, Artifact};
use crate::daemon::server::DaemonState;
use crate::fabric_context::ViewInputs;
use crate::reconcile::journal::{HookContextRenderFact, InputFact};
use anyhow::{Context, Result};
use std::sync::Arc;

pub(super) fn preview_session_start(
    state: &Arc<DaemonState>,
    fact: &InputFact,
) -> Result<Artifact> {
    let mut r = state
        .session_start
        .lock()
        .expect("session_start mutex poisoned");
    let preview = r
        .preview_fact(fact)
        .map_err(|e| anyhow::anyhow!("session_start preview failed: {e:?}"))?
        .context("probe: fact is not supported by session_start")?;
    Ok(plan_artifact(
        "session_start",
        &preview.labels,
        &preview.result,
        None,
    ))
}

pub(super) fn preview_session_watch(
    state: &Arc<DaemonState>,
    fact: &InputFact,
) -> Result<Artifact> {
    let mut r = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned");
    let preview = r
        .preview_fact(fact)
        .map_err(|e| anyhow::anyhow!("session_watch preview failed: {e:?}"))?
        .context("probe: fact is not supported by session_watch")?;
    Ok(plan_artifact(
        "session_watch",
        &preview.labels,
        &preview.result,
        None,
    ))
}

pub(super) fn preview_hook_context(state: &Arc<DaemonState>, fact: &InputFact) -> Result<Artifact> {
    let InputFact::HookContextRender(fact) = fact else {
        anyhow::bail!("probe: fact is not a hook_context fact");
    };
    let inputs = decode_hook_inputs(fact)?;
    let mut guard = state
        .hook_contexts
        .lock()
        .expect("hook-context mutex poisoned");
    let graph = guard.get_mut(&fact.session_id).with_context(|| {
        format!(
            "probe: hook_context graph for `{}` has not rendered",
            fact.session_id
        )
    })?;
    let preview = graph
        .preview_context(&fact.session_id, fact.cursor, fact.now, inputs)
        .map_err(|e| anyhow::anyhow!("hook_context preview failed: {e:?}"))?;
    Ok(plan_artifact(
        "hook_context",
        &preview.labels,
        &preview.result,
        None,
    ))
}

fn decode_hook_inputs(fact: &HookContextRenderFact) -> Result<ViewInputs> {
    serde_json::from_value(fact.inputs_json.clone()).context("decoding hook_context inputs")
}
