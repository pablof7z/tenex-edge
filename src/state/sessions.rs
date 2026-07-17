//! Pubkey-keyed durable session and runtime-incarnation persistence.

use super::*;
use rusqlite::{Transaction, TransactionBehavior};

pub(super) const COLS: &str =
    "pubkey, runtime_generation, agent_slug, channel_h, work_root, readiness_parent, \
     harness, child_pid, transcript_path, runtime_state, presentation_state, work_state, \
     recovery_state, lifecycle_epoch, attachment_epoch, idle_since, idle_deadline, \
     stopped_at, stop_reason, turn_count, created_at, last_seen, turn_started_at, \
     seen_cursor, title, explicit_chat_published_at";

fn conversion<T>(index: usize, result: Result<T>) -> rusqlite::Result<T> {
    result.map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(error.to_string())),
        )
    })
}

pub(super) fn row_to_session(row: &rusqlite::Row) -> rusqlite::Result<Session> {
    let runtime_state = conversion(9, RuntimeState::parse(&row.get::<_, String>(9)?))?;
    let presentation_state = conversion(10, PresentationState::parse(&row.get::<_, String>(10)?))?;
    let work_state = conversion(11, WorkState::parse(&row.get::<_, String>(11)?))?;
    let recovery_state = conversion(12, RecoveryState::parse(&row.get::<_, String>(12)?))?;
    let stop_reason = row
        .get::<_, Option<String>>(18)?
        .map(|value| conversion(18, StopReason::parse(&value)))
        .transpose()?;
    Ok(Session {
        pubkey: row.get(0)?,
        runtime_generation: row.get(1)?,
        agent_slug: row.get(2)?,
        channel_h: row.get(3)?,
        work_root: row.get(4)?,
        readiness_parent: row.get(5)?,
        harness: row.get(6)?,
        child_pid: row.get(7)?,
        transcript_path: row.get(8)?,
        runtime_state,
        presentation_state,
        work_state,
        recovery_state,
        lifecycle_epoch: row.get(13)?,
        attachment_epoch: row.get(14)?,
        idle_since: row.get(15)?,
        idle_deadline: row.get(16)?,
        stopped_at: row.get(17)?,
        stop_reason,
        turn_count: row.get(19)?,
        created_at: row.get(20)?,
        last_seen: row.get(21)?,
        turn_started_at: row.get(22)?,
        seen_cursor: row.get(23)?,
        title: row.get(24)?,
        explicit_chat_published_at: row.get(25)?,
    })
}

impl Store {
    /// Reserve the sole running incarnation. Stopping runtimes still own their
    /// pubkey; only a fully stopped runtime may advance the generation.
    pub fn reserve_session(&self, registration: &RegisterSession) -> Result<u64> {
        if registration.pubkey.trim().is_empty() {
            anyhow::bail!("session pubkey must not be empty");
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let previous = tx
            .query_row(
                "SELECT runtime_generation, runtime_state, recovery_state
                 FROM sessions WHERE pubkey=?1",
                [&registration.pubkey],
                |row| {
                    Ok((
                        row.get::<_, u64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        if let Some((_, state, recovery)) = previous.as_ref() {
            if RuntimeState::parse(state)? != RuntimeState::Stopped {
                anyhow::bail!(
                    "pubkey {} already has an active runtime",
                    registration.pubkey
                );
            }
            if RecoveryState::parse(recovery)? == RecoveryState::Revoked {
                anyhow::bail!(
                    "pubkey {} recovery authority is revoked",
                    registration.pubkey
                );
            }
        }
        let generation = match previous {
            Some((generation, _, _)) => generation
                .checked_add(1)
                .context("runtime generation exhausted")?,
            None => 1,
        };
        tx.execute(
            "INSERT INTO sessions
                 (pubkey, runtime_generation, agent_slug, channel_h, harness, child_pid,
                  transcript_path, runtime_state, presentation_state, work_state,
                  recovery_state, lifecycle_epoch, created_at, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'running', 'unavailable', 'idle',
                     'pending', 1, ?8, ?8)
             ON CONFLICT(pubkey) DO UPDATE SET
                 runtime_generation=excluded.runtime_generation,
                 agent_slug=excluded.agent_slug, channel_h=excluded.channel_h,
                 harness=excluded.harness, child_pid=excluded.child_pid,
                 transcript_path=excluded.transcript_path, runtime_state='running',
                 presentation_state='unavailable', work_state='idle',
                 lifecycle_epoch=sessions.lifecycle_epoch+1, attachment_epoch=0,
                 idle_since=0, idle_deadline=0, stopped_at=0, stop_reason=NULL,
                 created_at=excluded.created_at, last_seen=excluded.last_seen,
                 turn_started_at=0",
            params![
                registration.pubkey,
                generation,
                registration.agent_slug,
                registration.channel_h,
                registration.harness,
                registration.child_pid,
                registration.transcript_path,
                registration.now,
            ],
        )?;
        if !registration.channel_h.trim().is_empty() {
            grant_route_and_initialize_standing(&tx, registration)?;
        }
        tx.execute(
            "UPDATE handle_leases SET live=1, last_active_at=?2 WHERE pubkey=?1",
            params![registration.pubkey, registration.now],
        )?;
        tx.commit()?;
        Ok(generation)
    }

    pub fn get_session(&self, pubkey: &str) -> Result<Option<Session>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM sessions WHERE pubkey=?1"),
                [pubkey],
                row_to_session,
            )
            .optional()?)
    }

    pub(crate) fn session_exists(&self, pubkey: &str) -> Result<bool> {
        Ok(self.get_session(pubkey)?.is_some())
    }

    pub fn list_running_sessions(&self) -> Result<Vec<Session>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {COLS} FROM sessions WHERE runtime_state='running' ORDER BY created_at DESC"
        ))?;
        let rows = statement.query_map([], row_to_session)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_stopping_sessions(&self) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM sessions WHERE runtime_state='stopping' ORDER BY pubkey"
        ))?;
        let rows = stmt.query_map([], row_to_session)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn session_can_fresh_relaunch_exact(&self, pubkey: &str) -> Result<bool> {
        Ok(self
            .get_session(pubkey)?
            .is_some_and(|session| session.can_fresh_relaunch_exact()))
    }

    pub fn touch_session(&self, pubkey: &str, last_seen: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET last_seen=?2 WHERE pubkey=?1",
            params![pubkey, last_seen],
        )?;
        self.touch_handle_for_pubkey(pubkey, last_seen)
    }

    pub fn bind_runtime_process(
        &self,
        pubkey: &str,
        runtime_generation: u64,
        child_pid: Option<i32>,
    ) -> Result<bool> {
        Ok(self.conn.execute(
            "UPDATE sessions SET child_pid=?3, last_seen=?4
             WHERE pubkey=?1 AND runtime_generation=?2 AND runtime_state='running'
               AND recovery_state<>'revoked'",
            params![
                pubkey,
                runtime_generation,
                child_pid,
                crate::util::now_secs()
            ],
        )? > 0)
    }

    pub fn mark_runtime_stopped_if_generation(
        &self,
        pubkey: &str,
        generation: u64,
        reason: StopReason,
        stopped_at: u64,
    ) -> Result<bool> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE sessions
             SET runtime_state='stopped', presentation_state='unavailable', work_state='idle',
                 lifecycle_epoch=lifecycle_epoch+1, idle_since=0, idle_deadline=0,
                 stopped_at=?4, stop_reason=?3, turn_started_at=0
             WHERE pubkey=?1 AND runtime_generation=?2 AND runtime_state='running'",
            params![pubkey, generation, reason.as_str(), stopped_at],
        )?;
        if changed > 0 {
            mark_handle_stopped(&tx, pubkey)?;
            let lifecycle_epoch: u64 = tx.query_row(
                "SELECT lifecycle_epoch FROM sessions WHERE pubkey=?1",
                [pubkey],
                |row| row.get(0),
            )?;
            let retain_until =
                if matches!(reason, StopReason::AttachedCleanExit | StopReason::Revoked) {
                    stopped_at
                } else {
                    stopped_at.saturating_add(STOPPED_STANDING_RETENTION_SECS)
                };
            super::session_standing::retain_in_transaction(
                &tx,
                pubkey,
                lifecycle_epoch,
                retain_until,
                stopped_at,
            )?;
        }
        tx.commit()?;
        Ok(changed == 1)
    }

    pub fn mark_runtime_stopped(
        &self,
        pubkey: &str,
        reason: StopReason,
        stopped_at: u64,
    ) -> Result<bool> {
        let generation = self
            .get_session(pubkey)?
            .map(|session| session.runtime_generation);
        match generation {
            Some(generation) => {
                self.mark_runtime_stopped_if_generation(pubkey, generation, reason, stopped_at)
            }
            None => Ok(false),
        }
    }
}

fn grant_route_and_initialize_standing(
    tx: &Transaction<'_>,
    registration: &RegisterSession,
) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO session_channels (pubkey, channel_h, granted_at)
         VALUES (?1, ?2, ?3)",
        params![
            registration.pubkey,
            registration.channel_h,
            registration.now
        ],
    )?;
    tx.execute(
        "INSERT INTO session_standing
             (pubkey, channel_h, state, retain_until, standing_epoch,
              session_lifecycle_epoch, updated_at)
         SELECT ?1, ?2, 'absent', 0, 1, lifecycle_epoch, ?3
         FROM sessions WHERE pubkey=?1
         ON CONFLICT(pubkey, channel_h) DO NOTHING",
        params![
            registration.pubkey,
            registration.channel_h,
            registration.now
        ],
    )?;
    Ok(())
}

fn mark_handle_stopped(tx: &Transaction<'_>, pubkey: &str) -> Result<()> {
    tx.execute(
        "UPDATE handle_leases SET live=0,
             last_active_at=MAX(last_active_at,
                 COALESCE((SELECT last_seen FROM sessions WHERE pubkey=?1), 0))
         WHERE pubkey=?1",
        [pubkey],
    )?;
    Ok(())
}
