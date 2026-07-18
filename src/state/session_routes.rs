//! Durable exact-session channel affinity, independent of fabric standing.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmedAdmissionCommit {
    Committed,
    CleanupDue(SessionStanding),
    Superseded,
}

impl Store {
    pub fn commit_confirmed_session_admission(
        &self,
        pubkey: &str,
        channel_h: &str,
        runtime_generation: u64,
        lifecycle_epoch: u64,
        now: u64,
    ) -> Result<ConfirmedAdmissionCommit> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let owns = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions
             WHERE pubkey=?1 AND runtime_generation=?2 AND lifecycle_epoch=?3
               AND runtime_state='running' AND recovery_state<>'revoked')",
            params![pubkey, runtime_generation, lifecycle_epoch],
            |row| row.get::<_, bool>(0),
        )?;
        if !owns {
            let outcome = schedule_cleanup_in_transaction(
                &transaction,
                pubkey,
                channel_h,
                lifecycle_epoch,
                now,
            )?;
            transaction.commit()?;
            return self.finish_admission_outcome(pubkey, channel_h, outcome);
        }
        transaction.execute(
            "INSERT INTO session_channels (pubkey, channel_h, granted_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(pubkey, channel_h) DO UPDATE SET granted_at=excluded.granted_at",
            params![pubkey, channel_h, now],
        )?;
        transaction.execute(
            "INSERT INTO session_standing
                 (pubkey, channel_h, state, retain_until, standing_epoch,
                  session_lifecycle_epoch, updated_at)
             VALUES (?1, ?2, 'member', 0, 1, ?3, ?4)
             ON CONFLICT(pubkey, channel_h) DO UPDATE SET
                 state='member', retain_until=0,
                 standing_epoch=session_standing.standing_epoch+1,
                 session_lifecycle_epoch=excluded.session_lifecycle_epoch,
                 updated_at=excluded.updated_at",
            params![pubkey, channel_h, lifecycle_epoch, now],
        )?;
        transaction.commit()?;
        Ok(ConfirmedAdmissionCommit::Committed)
    }

    /// Persist compensation for relay admission whose primary commit failed.
    /// If the exact admission actually committed, or a newer lifecycle owns the
    /// member row, the result prevents a destructive stale removal.
    pub fn schedule_confirmed_admission_cleanup(
        &self,
        pubkey: &str,
        channel_h: &str,
        runtime_generation: u64,
        lifecycle_epoch: u64,
        now: u64,
    ) -> Result<ConfirmedAdmissionCommit> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let committed = transaction.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM sessions session
                 JOIN session_channels route ON route.pubkey=session.pubkey
                 JOIN session_standing standing ON standing.pubkey=session.pubkey
                 WHERE session.pubkey=?1 AND session.runtime_generation=?2
                   AND session.lifecycle_epoch=?3 AND session.runtime_state='running'
                   AND session.recovery_state<>'revoked'
                   AND route.channel_h=?4 AND standing.channel_h=?4
                   AND standing.state='member'
                   AND standing.session_lifecycle_epoch=?3
             )",
            params![pubkey, runtime_generation, lifecycle_epoch, channel_h],
            |row| row.get::<_, bool>(0),
        )?;
        if committed {
            transaction.rollback()?;
            return Ok(ConfirmedAdmissionCommit::Committed);
        }
        let outcome =
            schedule_cleanup_in_transaction(&transaction, pubkey, channel_h, lifecycle_epoch, now)?;
        transaction.commit()?;
        self.finish_admission_outcome(pubkey, channel_h, outcome)
    }

    fn finish_admission_outcome(
        &self,
        pubkey: &str,
        channel_h: &str,
        outcome: PendingAdmissionOutcome,
    ) -> Result<ConfirmedAdmissionCommit> {
        match outcome {
            PendingAdmissionOutcome::CleanupDue => Ok(ConfirmedAdmissionCommit::CleanupDue(
                self.get_session_standing(pubkey, channel_h)?
                    .context("scheduled admission cleanup row disappeared")?,
            )),
            PendingAdmissionOutcome::Superseded => Ok(ConfirmedAdmissionCommit::Superseded),
        }
    }

    pub fn revoke_route_and_mark_absent(
        &self,
        pubkey: &str,
        channel_h: &str,
        now: u64,
    ) -> Result<bool> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let removed = transaction.execute(
            "DELETE FROM session_channels WHERE pubkey=?1 AND channel_h=?2",
            params![pubkey, channel_h],
        )? > 0;
        transaction.execute(
            "INSERT INTO session_standing
                 (pubkey, channel_h, state, retain_until, standing_epoch,
                  session_lifecycle_epoch, updated_at)
             SELECT ?1, ?2, 'absent', 0, 1, lifecycle_epoch, ?3
               FROM sessions WHERE pubkey=?1
             ON CONFLICT(pubkey, channel_h) DO UPDATE SET
                 state='absent', retain_until=0,
                 standing_epoch=session_standing.standing_epoch+1,
                 session_lifecycle_epoch=excluded.session_lifecycle_epoch,
                 updated_at=excluded.updated_at",
            params![pubkey, channel_h, now],
        )?;
        transaction.commit()?;
        Ok(removed)
    }

    pub fn grant_session_route(
        &self,
        pubkey: &str,
        channel_h: &str,
        granted_at: u64,
    ) -> Result<()> {
        if channel_h.trim().is_empty() {
            return Ok(());
        }
        self.conn.execute(
            "INSERT OR IGNORE INTO session_channels (pubkey, channel_h, granted_at)
             VALUES (?1, ?2, ?3)",
            params![pubkey, channel_h, granted_at],
        )?;
        Ok(())
    }

    pub fn revoke_session_route(&self, pubkey: &str, channel_h: &str) -> Result<bool> {
        Ok(self.conn.execute(
            "DELETE FROM session_channels WHERE pubkey=?1 AND channel_h=?2",
            params![pubkey, channel_h],
        )? > 0)
    }

    pub fn has_session_route(&self, pubkey: &str, channel_h: &str) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM session_channels WHERE pubkey=?1 AND channel_h=?2
             )",
            params![pubkey, channel_h],
            |row| row.get(0),
        )?)
    }

    pub fn list_session_routes(&self, pubkey: &str) -> Result<Vec<(String, u64)>> {
        let mut statement = self.conn.prepare(
            "SELECT channel_h, granted_at FROM session_channels
             WHERE pubkey=?1 ORDER BY granted_at, channel_h",
        )?;
        let rows = statement.query_map([pubkey], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAdmissionOutcome {
    CleanupDue,
    Superseded,
}

fn schedule_cleanup_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    pubkey: &str,
    channel_h: &str,
    lifecycle_epoch: u64,
    now: u64,
) -> Result<PendingAdmissionOutcome> {
    let newer_member = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM session_standing
         WHERE pubkey=?1 AND channel_h=?2 AND state='member'
           AND session_lifecycle_epoch<>?3)",
        params![pubkey, channel_h, lifecycle_epoch],
        |row| row.get::<_, bool>(0),
    )?;
    if newer_member {
        return Ok(PendingAdmissionOutcome::Superseded);
    }
    transaction.execute(
        "INSERT INTO session_standing
             (pubkey, channel_h, state, retain_until, standing_epoch,
              session_lifecycle_epoch, updated_at)
         VALUES (?1, ?2, 'retained', ?4, 1, ?3, ?4)
         ON CONFLICT(pubkey, channel_h) DO UPDATE SET
             state='retained', retain_until=excluded.retain_until,
             standing_epoch=session_standing.standing_epoch+1,
             session_lifecycle_epoch=excluded.session_lifecycle_epoch,
             updated_at=excluded.updated_at",
        params![pubkey, channel_h, lifecycle_epoch, now],
    )?;
    Ok(PendingAdmissionOutcome::CleanupDue)
}

#[cfg(test)]
#[path = "session_routes/tests.rs"]
mod tests;
