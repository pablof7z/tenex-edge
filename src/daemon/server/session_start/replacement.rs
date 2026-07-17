use super::*;

/// A PID correlates host runtimes but never chooses public identity. When a new
/// native session starts on a watched PID, retire the prior pubkey generation
/// before the PID locator is rebound to the new owner.
pub(super) async fn retire_conflicting_pid_runtime(
    state: &Arc<DaemonState>,
    new_pubkey: &str,
    agent_slug: &str,
    harness: &str,
    watch_pid: Option<i32>,
    new_work_root: &str,
) -> Result<()> {
    let Some(pid) = watch_pid.map(|pid| pid.to_string()) else {
        return Ok(());
    };
    let Some(old_pubkey) = state.with_store(|store| {
        store.resolve_pubkey_by_locator(harness, crate::state::LOCATOR_PID, &pid)
    })?
    else {
        return Ok(());
    };
    if old_pubkey == new_pubkey {
        return Ok(());
    }
    let Some(old) = state.with_store(|store| store.get_session(&old_pubkey))? else {
        return Ok(());
    };
    if !old.is_running() || old.agent_slug != agent_slug {
        return Ok(());
    }
    let old_work_root = state
        .with_store(|store| store.root_channel_of(&old.channel_h).ok().flatten())
        .or_else(|| (!old.work_root.is_empty()).then(|| old.work_root.clone()))
        .unwrap_or_else(|| old.channel_h.clone());
    if old_work_root != new_work_root {
        return Ok(());
    }

    tracing::info!(
        old_pubkey,
        new_pubkey,
        runtime_generation = old.runtime_generation,
        pid,
        "retiring prior runtime generation on reused host pid"
    );
    super::super::managed_lifecycle::stop_generation(
        state,
        &old,
        crate::state::StopReason::Superseded,
        now_secs(),
    )
    .await?;
    Ok(())
}
