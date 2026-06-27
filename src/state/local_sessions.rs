use super::*;

impl Store {
    pub fn upsert_session(&self, r: &SessionRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions
               (session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
	             ON CONFLICT(session_id) DO UPDATE SET
	               agent_slug=?2, agent_pubkey=?3, project=?4, host=?5,
	               child_pid=?6, watch_pid=?7, alive=?9, rel_cwd=?10,
	               channel=CASE
	                 WHEN sessions.channel<>'' THEN sessions.channel
	                 ELSE excluded.channel
	               END",
            params![
                r.session_id, r.agent_slug, r.agent_pubkey, r.project, r.host,
                r.child_pid, r.watch_pid, r.created_at, r.alive as i32, r.rel_cwd, r.channel
            ],
        )?;
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<Option<SessionRecord>> {
        if let Some(rec) = self.get_session_exact(id)? {
            return Ok(Some(rec));
        }
        // Fallback: `id` may be a harness external id (claude/codex native id,
        // opencode resume token, tmux pane, watch pid). Resolve it to the canonical
        // session via `session_aliases`, newest mapping wins.
        let canonical: Option<String> = self
            .conn
            .query_row(
                "SELECT session_id FROM session_aliases
                 WHERE external_id=?1 ORDER BY created_at DESC LIMIT 1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .ok();
        match canonical {
            Some(canon) if canon != id => self.get_session_exact(&canon),
            _ => Ok(None),
        }
    }

    /// Resolve a possibly-aliased harness/external session id (or an already-
    /// canonical id) to the canonical `session_state` id. Hooks speak harness
    /// ids; every canonical transition (start_turn/end_turn/end_session/…) must
    /// be keyed by the minted canonical id or it silently updates zero rows.
    /// Returns the input unchanged when it is already canonical or has no alias
    /// mapping (so a brand-new id still flows through to registration).
    pub fn canonical_session_id(&self, id: &str) -> String {
        let is_canonical: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM session_state WHERE session_id=?1)",
                params![id],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if is_canonical {
            return id.to_string();
        }
        self.conn
            .query_row(
                "SELECT session_id FROM session_aliases
                 WHERE external_id=?1 ORDER BY created_at DESC LIMIT 1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .unwrap_or_else(|| id.to_string())
    }

    /// All locally-owned live sessions whose liveness is still fresh
    /// (`last_seen >= fresh_since`). Drives the daemon's heartbeat re-arm: every
    /// cadence these are re-published so the kind:30315 NIP-40 expiration is
    /// pushed forward and a live-but-idle session never ages off the relay.
    pub fn all_live_local_snapshots(&self, fresh_since: u64) -> Result<Vec<SessionSnapshot>> {
        let sql = format!(
            "SELECT {SESSION_STATE_COLS} FROM session_state
             WHERE lifecycle='active' AND last_seen>=?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![fresh_since], row_to_session_state)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Direct lookup of a session by its canonical id (no alias resolution).
    fn get_session_exact(&self, id: &str) -> Result<Option<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE session_id=?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_session(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_alive_sessions(&self) -> Result<Vec<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], row_to_session)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn list_local_agent_pubkeys(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT agent_pubkey FROM sessions")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Most-recent still-alive session for a project — lets an agent that
    /// doesn't know its session id resolve "my session" from the cwd.
    pub fn latest_alive_session_for_project(&self, project: &str) -> Result<Option<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 AND project=?1 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![project])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_session(row)?))
        } else {
            Ok(None)
        }
    }

    /// Most-recent still-alive session for a SPECIFIC agent in a project. Used by
    /// session resolution so a sender's identity is scoped to the invoking agent
    /// (`$TENEX_EDGE_AGENT`) rather than falling back to whatever agent was most
    /// recently active in the project — which would sign/record a `claude` send as
    /// `opencode` if opencode happened to be the latest-active session.
    pub fn latest_alive_session_for_agent_in_project(
        &self,
        agent_slug: &str,
        project: &str,
    ) -> Result<Option<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 AND project=?1 AND agent_slug=?2 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![project, agent_slug])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_session(row)?))
        } else {
            Ok(None)
        }
    }

    /// Most-recent still-alive session for an agent under a `work_root` — the
    /// bare project OR any per-session room minted beneath it
    /// (`session-<hash>`). A human-initiated session is stored under its minted
    /// room (issue #6), but the same terminal's later `tenex-edge` verbs only
    /// resolve the bare work-root from `cwd` (no `TENEX_EDGE_CHANNEL` is exported
    /// into an already-running interactive shell). Without this the agent can't
    /// find the very session it is running inside. Pass `None` for `agent` to
    /// match any agent in the work-root.
    pub fn latest_alive_session_under_work_root(
        &self,
        work_root: &str,
        agent_slug: Option<&str>,
    ) -> Result<Option<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 AND (?2 IS NULL OR agent_slug=?2) ORDER BY created_at DESC",
        )?;
        let mut rows = stmt.query(params![work_root, agent_slug])?;
        while let Some(row) = rows.next()? {
            let rec = row_to_session(row)?;
            // The session is either directly in the bare work-root project, or in
            // a per-session room nested under it. The room id (session-<hash>) no
            // longer encodes the project, so match on the stored room_parent.
            let under_work_root = rec.project == work_root
                || self.session_room_parent(&rec.project)?.as_deref() == Some(work_root);
            if under_work_root {
                return Ok(Some(rec));
            }
        }
        Ok(None)
    }

    /// Persist the harness-native resume token for a session. Idempotent; a
    /// later call with the same token is a no-op. Never clears a known token with
    /// an empty one (so a stray payload can't wipe a good resume id).
    pub fn set_session_resume_id(&self, session_id: &str, resume_id: &str) -> Result<()> {
        if resume_id.is_empty() {
            return Ok(());
        }
        self.conn.execute(
            "UPDATE sessions SET resume_id=?2 WHERE session_id=?1",
            params![session_id, resume_id],
        )?;
        Ok(())
    }

    /// The harness-native resume token for a session, or `None` when unset/empty.
    pub fn get_session_resume_id(&self, session_id: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT resume_id FROM sessions WHERE session_id=?1",
                params![session_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .filter(|s| !s.is_empty()))
    }

    /// Recent sessions on `host`, newest first, with their stored `resume_id`
    /// (which may be empty — claude/codex sessions use their `session_id` as the
    /// resume token, so the caller derives the real token rather than filtering
    /// here). Includes dead (`alive=0`) rows. Returns `(record, resume_id)` pairs.
    pub fn list_resumable_sessions(
        &self,
        host: &str,
        limit: usize,
    ) -> Result<Vec<(SessionRecord, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel, resume_id
             FROM sessions WHERE host=?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![host, limit as i64], |row| {
            let rec = row_to_session(row)?;
            let resume_id: String = row.get(11)?;
            Ok((rec, resume_id))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn mark_session_dead(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET alive=0 WHERE session_id=?1",
            params![id],
        )?;
        Ok(())
    }

    /// Move a session to a different NIP-29 routing scope (`channels switch`).
    /// Writes `sessions.channel` AND `session_state.project` so the status
    /// drainer, `who`/`statusline` scoping, and `status_delta_since` all key on
    /// the new scope. Bumping `session_state.project` + `state_version` +
    /// enqueueing an outbox row makes the next kind:30315 publish land in the
    /// new group, so peers see the session's heartbeat under the channel it
    /// switched to rather than the per-session room it minted at spawn. `scope`
    /// is the new NIP-29 group id (channel); pass `""` to clear the binding and
    /// fall back to the per-session room (`sessions.project`).
    pub fn set_session_channel(&self, session_id: &str, scope: &str, ts: u64) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<()> {
            // The effective routing scope: the new channel when non-empty, else the
            // per-session room (`sessions.project`) — `channels switch ""` reverts.
            let effective: String = if scope.is_empty() {
                self.conn.query_row(
                    "SELECT project FROM sessions WHERE session_id=?1",
                    params![session_id],
                    |r| r.get::<_, String>(0),
                )?
            } else {
                scope.to_string()
            };
            // sessions.channel tracks the user-facing channel binding (empty = none).
            let session_rows = self.conn.execute(
                "UPDATE sessions SET channel=?2 WHERE session_id=?1",
                params![session_id, scope],
            )?;
            if session_rows != 1 {
                anyhow::bail!("unknown session {session_id}");
            }
            // session_state.project is the routing scope the drainer + who/turn
            // deltas read; move it to the effective scope and bump the version so a
            // fresh kind:30315 is enqueued for the new group.
            let state_rows = self.conn.execute(
                "UPDATE session_state SET project=?2, state_version=state_version+1, updated_at=?3, last_seen=?3
                 WHERE session_id=?1",
                params![session_id, effective, ts],
            )?;
            if state_rows != 1 {
                anyhow::bail!("missing session_state row for session {session_id}");
            }
            self.enqueue_status_outbox_current(session_id, ts)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Record the host transcript path for a session (provided by the hook), so
    /// the engine can read the recent conversation to distill activity.
    pub fn set_session_transcript(&self, id: &str, path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET transcript_path=?2 WHERE session_id=?1",
            params![id, path],
        )?;
        Ok(())
    }

    pub fn get_session_transcript(&self, id: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT transcript_path FROM sessions WHERE session_id=?1",
                params![id],
                |r| r.get::<_, Option<String>>(0),
            )
            .ok()
            .flatten())
    }

    /// Snapshot the last assistant text at the start of a turn. `rpc_turn_end`
    /// polls until the transcript returns something *different* from this value,
    /// so it reliably reads the current turn's response even when Claude Code
    /// writes the transcript after the stop hook fires.
    pub fn set_last_assistant_text_at_turn_start(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET last_assistant_text_at_turn_start=?2 WHERE session_id=?1",
            params![session_id, text],
        )?;
        Ok(())
    }

    pub fn get_last_assistant_text_at_turn_start(&self, session_id: &str) -> String {
        self.conn
            .query_row(
                "SELECT last_assistant_text_at_turn_start FROM sessions WHERE session_id=?1",
                params![session_id],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_default()
    }

    /// Heartbeat: keep a live session's `last_seen` fresh (called each engine tick).
    pub fn touch_session(&self, id: &str, ts: u64) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET last_seen=?2 WHERE session_id=?1",
            params![id, ts],
        )?;
        Ok(())
    }

    pub fn session_last_seen(&self, id: &str) -> Result<Option<u64>> {
        Ok(self
            .conn
            .query_row(
                "SELECT last_seen FROM sessions WHERE session_id=?1",
                params![id],
                |r| r.get::<_, u64>(0),
            )
            .ok())
    }

    /// My own sessions whose heartbeat is fresh (alive + recently touched).
    pub fn list_my_live_sessions(&self, since: u64) -> Result<Vec<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 AND last_seen>=?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![since], row_to_session)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
