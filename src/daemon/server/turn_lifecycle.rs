use super::*;

pub(in crate::daemon::server) fn seed_from_session(
    rec: &crate::state::Session,
) -> crate::reconcile::TurnProjectionSeed {
    crate::reconcile::TurnProjectionSeed {
        session_id: rec.session_id.clone(),
        working: rec.working,
        turn_started_at: rec.turn_started_at,
        transcript_ref: rec.transcript_path.clone(),
    }
}

pub(in crate::daemon::server) fn drive_turn_started(
    state: &Arc<DaemonState>,
    seed: crate::reconcile::TurnProjectionSeed,
    at: u64,
    transcript_ref: Option<String>,
) -> Result<()> {
    let mut facts = vec![crate::reconcile::InputFact::TurnStarted {
        session_id: seed.session_id.clone(),
        at,
    }];
    if let Some(window_hash) = transcript_ref.clone() {
        facts.push(crate::reconcile::InputFact::TranscriptWindowCaptured {
            session_id: seed.session_id.clone(),
            window_hash,
            at,
        });
    }
    drive_projection(state, "turn_started", seed.clone(), facts, |r| {
        let preview = r.preview_turn_started(seed.clone(), at, transcript_ref.clone())?;
        let outcome = r.on_turn_started(seed, at, transcript_ref)?;
        Ok((preview.result, outcome))
    })
}

pub(in crate::daemon::server) fn drive_turn_ended(
    state: &Arc<DaemonState>,
    seed: crate::reconcile::TurnProjectionSeed,
    at: u64,
) -> Result<()> {
    let facts = vec![crate::reconcile::InputFact::TurnEnded {
        session_id: seed.session_id.clone(),
        at,
    }];
    drive_projection(state, "turn_ended", seed.clone(), facts, |r| {
        let preview = r.preview_turn_ended(seed.clone(), at)?;
        let outcome = r.on_turn_ended(seed, at)?;
        Ok((preview.result, outcome))
    })
}

fn drive_projection(
    state: &Arc<DaemonState>,
    trigger: &str,
    seed: crate::reconcile::TurnProjectionSeed,
    facts: Vec<crate::reconcile::InputFact>,
    f: impl FnOnce(
        &mut crate::reconcile::TurnLifecycleReconciler,
    ) -> trellis_core::GraphResult<(
        trellis_core::TransactionResult<crate::reconcile::TurnCommand>,
        crate::reconcile::TurnLifecycleOutcome,
    )>,
) -> Result<()> {
    let start = std::time::Instant::now();
    let (preview, outcome, commit) = {
        let mut r = state
            .turn_lifecycle
            .lock()
            .expect("turn lifecycle mutex poisoned");
        let (preview, outcome) =
            f(&mut r).map_err(|e| anyhow::anyhow!("turn lifecycle drive failed: {e:?}"))?;
        let mut commit = crate::reconcile::CommitFacts::from_result(
            r.labels(),
            &outcome.result,
            r.graph_node_count(),
        );
        commit.graph_resources = r.state_rows().len() as i64;
        (preview, outcome, commit)
    };
    if !outcome.effects.is_empty()
        && !crate::reconcile::preview::command_plans_match(
            preview.resource_plan.commands(),
            outcome.result.resource_plan.commands(),
        )
    {
        anyhow::bail!("turn lifecycle effects blocked: committed plan was not previewed first");
    }
    apply_effects(state, outcome.effects)?;
    let created_at = crate::instrument::now_millis();
    let duration_us = start.elapsed().as_micros() as i64;
    state.with_store(|s| {
        crate::instrument::record_commit(
            s,
            "turn_lifecycle",
            trigger,
            Some(seed.session_id.as_str()),
            &commit,
            duration_us,
            created_at,
        );
        crate::replay_capsules::record_many(
            s,
            "turn_lifecycle",
            trigger,
            Some(seed.session_id.as_str()),
            facts,
            created_at,
        );
    });
    Ok(())
}

fn apply_effects(
    state: &Arc<DaemonState>,
    effects: Vec<crate::reconcile::TurnEffect>,
) -> Result<()> {
    for effect in effects {
        match effect {
            crate::reconcile::TurnEffect::Apply(cmd) => state.with_store(|s| {
                s.apply_turn_projection(
                    &cmd.session_id,
                    cmd.working,
                    cmd.turn_started_at,
                    cmd.transcript_ref.as_deref(),
                )
            })?,
        }
    }
    Ok(())
}
