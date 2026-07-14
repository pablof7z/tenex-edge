use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ALIAS_ATTEMPT: AtomicU64 = AtomicU64::new(1);

pub(super) struct AliasRollbackGuard {
    state: Arc<DaemonState>,
    owner: String,
    armed: bool,
}

impl AliasRollbackGuard {
    fn new(state: &Arc<DaemonState>, harness: &str) -> Self {
        Self {
            state: state.clone(),
            owner: format!(
                "{}:{}:{harness}",
                std::process::id(),
                NEXT_ALIAS_ATTEMPT.fetch_add(1, Ordering::Relaxed)
            ),
            armed: true,
        }
    }

    pub(super) fn disarm(&mut self) {
        if let Err(error) = self
            .state
            .with_store(|store| store.commit_alias_attempt(&self.owner))
        {
            tracing::error!(owner = %self.owner, %error, "failed to commit session-start aliases");
        }
        self.armed = false;
    }
}

impl Drop for AliasRollbackGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Err(error) = self
            .state
            .with_store(|store| store.abort_alias_attempt(&self.owner))
        {
            tracing::error!(
                owner = %self.owner,
                %error,
                "failed to restore aliases after aborted session start"
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_session_id(
    state: &Arc<DaemonState>,
    harness: &str,
    pty_session: Option<&str>,
    harness_session_id: Option<&str>,
    resume_id: Option<&str>,
    watch_pid: Option<i32>,
    durable_agent: bool,
    now: u64,
) -> Result<(String, &'static str, String, AliasRollbackGuard)> {
    let guard = AliasRollbackGuard::new(state, harness);
    // Canonical identity: the daemon MINTS a stable session id; the harness id /
    // resume token / endpoint / pid become rows in `session_aliases`. For
    // PTY-hosted launches, a delayed harness hook must reassert the spawn-time
    // PTY registration instead of minting a second row under its native id.
    let (session_id, ext_kind, ext_id) = state.with_store(|store| {
        let existing_pty_session = match pty_session {
            Some(pty) => store
                .alive_session_for_alias(None, "pty_session", pty)?
                .map(|session| session.session_id),
            None => None,
        };
        let (ext_kind, ext_id) = match (
            existing_pty_session.as_ref(),
            pty_session,
            harness_session_id,
            resume_id,
            watch_pid,
        ) {
            (Some(_), Some(pty), _, _, _) => ("pty_session", pty.to_string()),
            (None, _, Some(hs), _, _) => ("harness_session", hs.to_string()),
            (None, _, None, Some(resume), _) => ("resume", resume.to_string()),
            (None, _, None, None, Some(pid)) => ("watch_pid", pid.to_string()),
            (None, Some(pty), None, None, None) => ("pty_session", pty.to_string()),
            _ => ("harness_session", String::new()),
        };
        let session_id = match existing_pty_session {
            Some(session_id) => session_id,
            None => select_session_id(store, harness, ext_kind, &ext_id, durable_agent)?,
        };
        store.put_alias_provisional(harness, ext_kind, &ext_id, &session_id, now, &guard.owner)?;
        Ok::<_, anyhow::Error>((session_id, ext_kind, ext_id))
    })?;
    Ok((session_id, ext_kind, ext_id, guard))
}

fn select_session_id(
    store: &crate::state::Store,
    harness: &str,
    kind: &str,
    external_id: &str,
    durable_agent: bool,
) -> Result<String> {
    let prior = store.resolve_session_by_alias(harness, kind, external_id)?;
    if let Some(session_id) = prior {
        let session = store.session_row(&session_id)?;
        if session.is_some_and(|session| !durable_agent || session.alive) {
            return Ok(session_id);
        }
    }
    Ok(crate::state::mint_session_id())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_secondary_aliases(
    guard: &AliasRollbackGuard,
    harness: &str,
    session_id: &str,
    pty_session: Option<&str>,
    harness_session_id: Option<&str>,
    resume_id: Option<&str>,
    watch_pid: Option<i32>,
    work_root: &str,
    cwd: &std::path::Path,
    channel: &str,
    now: u64,
) {
    let state = guard.state.clone();
    let owner = guard.owner.clone();
    state.with_store(|s| {
        for (kind, value) in [
            ("pty_session", pty_session),
            ("harness_session", harness_session_id),
            ("resume", resume_id),
        ] {
            if let Some(value) = value {
                s.put_alias_provisional(harness, kind, value, session_id, now, &owner)
                    .ok();
            }
        }
        if let Some(pid) = watch_pid {
            let value = pid.to_string();
            s.put_alias_provisional(harness, "watch_pid", &value, session_id, now, &owner)
                .ok();
        }
        s.upsert_workspace(work_root, &cwd.to_string_lossy(), now)
            .ok();
        if channel != work_root {
            s.upsert_workspace(channel, &cwd.to_string_lossy(), now)
                .ok();
        }
    });
}

#[cfg(test)]
#[path = "alias_resolution/tests.rs"]
mod tests;
