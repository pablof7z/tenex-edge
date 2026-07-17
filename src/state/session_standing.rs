use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandingState {
    Member,
    Retained,
    Absent,
}

impl StandingState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Retained => "retained",
            Self::Absent => "absent",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "member" => Ok(Self::Member),
            "retained" => Ok(Self::Retained),
            "absent" => Ok(Self::Absent),
            _ => anyhow::bail!("unknown StandingState value {value:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStanding {
    pub pubkey: String,
    pub channel_h: String,
    pub state: StandingState,
    pub retain_until: u64,
    pub standing_epoch: u64,
    pub session_lifecycle_epoch: u64,
    pub updated_at: u64,
}

const COLS: &str = "pubkey, channel_h, state, retain_until, standing_epoch, \
                    session_lifecycle_epoch, updated_at";

fn row_to_standing(row: &rusqlite::Row) -> rusqlite::Result<SessionStanding> {
    let raw: String = row.get(2)?;
    let state = StandingState::parse(&raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(error.to_string())),
        )
    })?;
    Ok(SessionStanding {
        pubkey: row.get(0)?,
        channel_h: row.get(1)?,
        state,
        retain_until: row.get(3)?,
        standing_epoch: row.get(4)?,
        session_lifecycle_epoch: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

impl Store {
    pub fn get_session_standing(
        &self,
        pubkey: &str,
        channel_h: &str,
    ) -> Result<Option<SessionStanding>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM session_standing WHERE pubkey=?1 AND channel_h=?2"),
                params![pubkey, channel_h],
                row_to_standing,
            )
            .optional()?)
    }

    pub fn list_session_standing(&self, pubkey: &str) -> Result<Vec<SessionStanding>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {COLS} FROM session_standing WHERE pubkey=?1 ORDER BY channel_h"
        ))?;
        let rows = statement.query_map([pubkey], row_to_standing)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Apply confirmed relay admission only while the same running lifecycle
    /// still owns the request. A stale channel-ready completion returns `None`.
    pub fn mark_session_standing_member_if_running(
        &self,
        pubkey: &str,
        channel_h: &str,
        expected_lifecycle_epoch: u64,
        now: u64,
    ) -> Result<Option<u64>> {
        let changed = self.conn.execute(
            "INSERT INTO session_standing
                 (pubkey, channel_h, state, retain_until, standing_epoch,
                  session_lifecycle_epoch, updated_at)
             SELECT ?1, ?2, 'member', 0, 1, lifecycle_epoch, ?4
             FROM sessions
             WHERE pubkey=?1 AND runtime_state='running' AND lifecycle_epoch=?3
               AND recovery_state<>'revoked'
             ON CONFLICT(pubkey, channel_h) DO UPDATE SET
                 state='member', retain_until=0,
                 standing_epoch=session_standing.standing_epoch + 1,
                 session_lifecycle_epoch=excluded.session_lifecycle_epoch,
                 updated_at=excluded.updated_at
             WHERE EXISTS (
                 SELECT 1 FROM sessions
                 WHERE pubkey=?1 AND runtime_state='running' AND lifecycle_epoch=?3
                   AND recovery_state<>'revoked'
             )",
            params![pubkey, channel_h, expected_lifecycle_epoch, now],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(Some(self.conn.query_row(
            "SELECT standing_epoch FROM session_standing WHERE pubkey=?1 AND channel_h=?2",
            params![pubkey, channel_h],
            |row| row.get(0),
        )?))
    }

    /// Move every currently-member route to retained standing, fenced by the
    /// stopped session lifecycle that created the one-hour deadline.
    pub fn retain_stopped_session_routes(
        &self,
        pubkey: &str,
        expected_lifecycle_epoch: u64,
        retain_until: u64,
        now: u64,
    ) -> Result<Option<Vec<SessionStanding>>> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.conn,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let owns = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions
             WHERE pubkey=?1 AND runtime_state='stopped' AND lifecycle_epoch=?2)",
            params![pubkey, expected_lifecycle_epoch],
            |row| row.get::<_, bool>(0),
        )?;
        if !owns {
            transaction.rollback()?;
            return Ok(None);
        }
        transaction.execute(
            "INSERT INTO session_standing
                 (pubkey, channel_h, state, retain_until, standing_epoch,
                  session_lifecycle_epoch, updated_at)
             SELECT route.pubkey, route.channel_h, 'absent', 0, 1, ?2, ?3
             FROM session_channels route WHERE route.pubkey=?1
             ON CONFLICT(pubkey, channel_h) DO NOTHING",
            params![pubkey, expected_lifecycle_epoch, now],
        )?;
        transaction.execute(
            "UPDATE session_standing
             SET state='retained', retain_until=?3, standing_epoch=standing_epoch+1,
                 session_lifecycle_epoch=?2, updated_at=?4
             WHERE pubkey=?1 AND state='member'",
            params![pubkey, expected_lifecycle_epoch, retain_until, now],
        )?;
        transaction.commit()?;
        Ok(Some(self.list_session_standing(pubkey)?))
    }

    pub fn list_due_retained_standing(&self, now: u64) -> Result<Vec<SessionStanding>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {COLS} FROM session_standing
             WHERE state='retained' AND retain_until>0 AND retain_until<=?1
             ORDER BY retain_until, pubkey, channel_h"
        ))?;
        let rows = statement.query_map([now], row_to_standing)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_retained_session_standing(&self, now: u64) -> Result<Vec<SessionStanding>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {COLS} FROM session_standing
             WHERE state='retained' AND retain_until>?1
             ORDER BY updated_at DESC, pubkey, channel_h"
        ))?;
        let rows = statement.query_map([now], row_to_standing)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn mark_session_standing_absent_if_epoch(
        &self,
        pubkey: &str,
        channel_h: &str,
        expected_state: StandingState,
        standing_epoch: u64,
        session_lifecycle_epoch: u64,
        now: u64,
    ) -> Result<bool> {
        Ok(self.conn.execute(
            "UPDATE session_standing
             SET state='absent', retain_until=0, standing_epoch=standing_epoch+1,
                 updated_at=?6
             WHERE pubkey=?1 AND channel_h=?2 AND state=?3
               AND standing_epoch=?4 AND session_lifecycle_epoch=?5",
            params![
                pubkey,
                channel_h,
                expected_state.as_str(),
                standing_epoch,
                session_lifecycle_epoch,
                now
            ],
        )? == 1)
    }
}

pub(super) fn retain_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    pubkey: &str,
    lifecycle_epoch: u64,
    retain_until: u64,
    now: u64,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO session_standing
             (pubkey, channel_h, state, retain_until, standing_epoch,
              session_lifecycle_epoch, updated_at)
         SELECT route.pubkey, route.channel_h, 'absent', 0, 1, ?2, ?3
         FROM session_channels route WHERE route.pubkey=?1
         ON CONFLICT(pubkey, channel_h) DO NOTHING",
        params![pubkey, lifecycle_epoch, now],
    )?;
    transaction.execute(
        "UPDATE session_standing
         SET state='retained', retain_until=?3, standing_epoch=standing_epoch+1,
             session_lifecycle_epoch=?2, updated_at=?4
         WHERE pubkey=?1 AND state='member'",
        params![pubkey, lifecycle_epoch, retain_until, now],
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "session_standing/tests.rs"]
mod tests;
