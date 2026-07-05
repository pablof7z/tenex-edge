use super::*;

pub(in crate::daemon::server) fn seed_from_session(
    rec: &crate::state::Session,
) -> crate::reconcile::CursorSeed {
    crate::reconcile::CursorSeed {
        session_id: rec.session_id.clone(),
        seen_cursor: rec.seen_cursor,
    }
}

pub(in crate::daemon::server) fn fact_from_session(
    rec: &crate::state::Session,
    at: u64,
    working: bool,
) -> crate::reconcile::InputFact {
    crate::reconcile::InputFact::TurnCheckRequested {
        session_id: rec.session_id.clone(),
        observed_cursor: rec.seen_cursor,
        working,
        at,
    }
}

pub(in crate::daemon::server) fn drive_cursor_request(
    state: &Arc<DaemonState>,
    trigger: &str,
    seed: crate::reconcile::CursorSeed,
    fact: crate::reconcile::InputFact,
) -> Result<Option<u64>> {
    let start = std::time::Instant::now();
    let facts = vec![fact.clone()];
    let (preview, outcome, commit) = {
        let mut r = state.cursor.lock().expect("cursor mutex poisoned");
        let preview = r
            .preview_request(seed.clone(), &fact)
            .map_err(|e| anyhow::anyhow!("cursor preview failed: {e:?}"))?;
        let outcome = r
            .request(seed.clone(), fact)
            .map_err(|e| anyhow::anyhow!("cursor drive failed: {e:?}"))?;
        let mut commit = crate::reconcile::CommitFacts::from_result(
            r.labels(),
            &outcome.result,
            r.graph_node_count(),
        );
        commit.graph_resources = r.state_rows().len() as i64;
        (preview.result, outcome, commit)
    };
    if !crate::reconcile::preview::command_plans_match(
        preview.resource_plan.commands(),
        outcome.result.resource_plan.commands(),
    ) {
        anyhow::bail!("cursor effects blocked: committed plan was not previewed first");
    }
    let delta_since = apply_effects(state, outcome.effects)?;
    let created_at = crate::instrument::now_millis();
    let duration_us = start.elapsed().as_micros() as i64;
    state.with_store(|s| {
        crate::instrument::record_commit(
            s,
            "cursor",
            trigger,
            Some(seed.session_id.as_str()),
            &commit,
            duration_us,
            created_at,
        );
        crate::replay_capsules::record_many(
            s,
            "cursor",
            trigger,
            Some(seed.session_id.as_str()),
            facts,
            created_at,
        );
    });
    Ok(delta_since)
}

fn apply_effects(
    state: &Arc<DaemonState>,
    effects: Vec<crate::reconcile::CursorEffect>,
) -> Result<Option<u64>> {
    let mut delta_since = None;
    for effect in effects {
        match effect {
            crate::reconcile::CursorEffect::Advance {
                session_id,
                to,
                delta_since: since,
                ..
            } => {
                state.with_store(|s| s.apply_cursor_projection(&session_id, to))?;
                delta_since = Some(since);
            }
            crate::reconcile::CursorEffect::NoFrame => {}
        }
    }
    Ok(delta_since)
}
