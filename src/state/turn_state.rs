use super::*;

impl Store {
    pub fn mark_turn_start(&self, session_id: &str, ts: u64) -> Result<()> {
        // Reset the mid-turn delta cursor (last_check_at=0) so the first
        // PostToolUse of the new turn reports sibling changes since `ts`.
        self.conn.execute(
            "INSERT INTO turn_state (session_id, working, turn_started_at, last_check_at) VALUES (?1, 1, ?2, 0)
             ON CONFLICT(session_id) DO UPDATE SET working=1, turn_started_at=?2, last_check_at=0",
            params![session_id, ts],
        )?;
        Ok(())
    }

    /// Mark a session idle (the turn ended). The engine publishes idle status on
    /// its next poll.
    pub fn mark_turn_end(&self, session_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO turn_state (session_id, working, turn_started_at, last_check_at) VALUES (?1, 0, 0, 0)
             ON CONFLICT(session_id) DO UPDATE SET working=0, last_check_at=0",
            params![session_id],
        )?;
        Ok(())
    }

    /// Mid-turn delta gate for PostToolUse `turn_check`. If the session is in a
    /// turn AND at least `min_interval` seconds have passed since the last check
    /// (or since turn start, if no check yet this turn), advance the cursor to
    /// `now` and return `Some(since)` — the timestamp to query sibling-session
    /// deltas from. Returns `None` when not in a turn or rate-limited, so the
    /// hook stays silent. The write is safe here: `turn_check` is daemon-
    /// mediated, so the daemon's single store connection is the only writer.
    pub fn turn_check_due(
        &self,
        session_id: &str,
        now: u64,
        min_interval: u64,
    ) -> Result<Option<u64>> {
        let (working, turn_started_at, last_check_at) = self
            .conn
            .query_row(
                "SELECT working, turn_started_at, last_check_at FROM turn_state WHERE session_id=?1",
                params![session_id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)? != 0,
                        r.get::<_, i64>(1)? as u64,
                        r.get::<_, i64>(2)? as u64,
                    ))
                },
            )
            .unwrap_or((false, 0, 0));
        // Only mid-turn (turn_end leaves turn_started_at set but clears working).
        // The turn_started_at guard also avoids querying all history pre-turn.
        if !working || turn_started_at == 0 {
            return Ok(None);
        }
        let since = if last_check_at > 0 {
            // Subsequent check this turn: enforce the floor against the last one.
            if now.saturating_sub(last_check_at) < min_interval {
                return Ok(None);
            }
            last_check_at
        } else {
            // First check of the turn → always due; window opens at turn start.
            turn_started_at
        };
        self.conn.execute(
            "UPDATE turn_state SET last_check_at=?2 WHERE session_id=?1",
            params![session_id, now],
        )?;
        Ok(Some(since))
    }

    /// `(working, turn_started_at)` for a session. Defaults to `(false, 0)` when
    /// no turn has started yet, so the engine simply stays idle.
    pub fn get_turn_state(&self, session_id: &str) -> Result<(bool, u64)> {
        Ok(self
            .conn
            .query_row(
                "SELECT working, turn_started_at FROM turn_state WHERE session_id=?1",
                params![session_id],
                |r| Ok((r.get::<_, i64>(0)? != 0, r.get::<_, i64>(1)? as u64)),
            )
            .unwrap_or((false, 0)))
    }

    /// Returns `true` if the session is currently mid-turn (`working = 1`).
    /// Defaults to `false` (not working) when no row exists, so tmux injection is
    /// allowed when a session has never started a turn.
    pub fn is_session_working(&self, session_id: &str) -> bool {
        self.conn
            .query_row(
                "SELECT working FROM turn_state WHERE session_id=?1",
                params![session_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            != 0
    }

    /// Count undelivered chat rows that explicitly mention this session.
    pub fn count_unread_chat_mentions(&self, session_id: &str) -> Result<usize> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM chat_inbox
             WHERE target_session=?1 AND mentioned_session=?1 AND delivered=0",
            params![session_id],
            |r| r.get(0),
        )?;
        Ok(n as usize)
    }
}
