use super::{plan_value, PlanJson};
use crate::daemon::server::DaemonState;
use crate::fabric_context::ViewInputs;
use crate::reconcile::journal::{HookContextRenderFact, InputFact};
use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;

pub(super) fn simulate_session_start_fact(
    state: &Arc<DaemonState>,
    fact: InputFact,
) -> Result<Value> {
    let mut r = state
        .session_start
        .lock()
        .expect("session_start mutex poisoned");
    let revision_before = r.revision();
    let preview = r
        .preview_fact(&fact)
        .map_err(|e| anyhow::anyhow!("session_start preview failed: {e:?}"))?
        .context("probe simulate: fact is not supported by session_start")?;
    let revision_after = r.revision();
    Ok(plan_value(PlanJson {
        surface: "session_start",
        fact: serde_json::to_value(&fact).unwrap_or(Value::Null),
        labels: &preview.labels,
        plan: &preview.result,
        revision_before,
        revision_after,
        wire_kind: None,
        would_publish: None,
    }))
}

pub(super) fn simulate_session_watch_fact(
    state: &Arc<DaemonState>,
    fact: InputFact,
) -> Result<Value> {
    let mut r = state
        .session_watch
        .lock()
        .expect("session_watch mutex poisoned");
    let revision_before = r.revision();
    let preview = r
        .preview_fact(&fact)
        .map_err(|e| anyhow::anyhow!("session_watch preview failed: {e:?}"))?
        .context("probe simulate: fact is not supported by session_watch")?;
    let revision_after = r.revision();
    Ok(plan_value(PlanJson {
        surface: "session_watch",
        fact: serde_json::to_value(&fact).unwrap_or(Value::Null),
        labels: &preview.labels,
        plan: &preview.result,
        revision_before,
        revision_after,
        wire_kind: None,
        would_publish: None,
    }))
}

pub(super) fn simulate_hook_context_fact(
    state: &Arc<DaemonState>,
    fact: InputFact,
) -> Result<Value> {
    let InputFact::HookContextRender(render) = &fact else {
        anyhow::bail!("probe simulate: fact is not a hook_context fact");
    };
    let inputs = decode_hook_inputs(render)?;
    let mut guard = state
        .hook_contexts
        .lock()
        .expect("hook-context mutex poisoned");
    let graph = guard.get_mut(&render.session_id).with_context(|| {
        format!(
            "probe simulate: hook_context graph for `{}` has not rendered",
            render.session_id
        )
    })?;
    let revision_before = graph.revision();
    let preview_result = graph
        .preview_context(&render.session_id, render.cursor, render.now, inputs)
        .map(|preview| (preview, revision_before, graph.revision()));
    let (preview, revision_before, revision_after) =
        preview_result.map_err(|e| anyhow::anyhow!("hook_context preview failed: {e:?}"))?;
    Ok(plan_value(PlanJson {
        surface: "hook_context",
        fact: serde_json::to_value(&fact).unwrap_or(Value::Null),
        labels: &preview.labels,
        plan: &preview.result,
        revision_before,
        revision_after,
        wire_kind: None,
        would_publish: None,
    }))
}

fn decode_hook_inputs(fact: &HookContextRenderFact) -> Result<ViewInputs> {
    serde_json::from_value(fact.inputs_json.clone()).context("decoding hook_context inputs")
}
