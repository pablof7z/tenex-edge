use super::*;

impl Store {
    pub fn start_turn(&self, session_id: &str, ts: u64) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET
               busy=1, phase='working', activity='',
               turn_id=turn_id+1, turn_started_at=?2,
               state_version=state_version+1, updated_at=?2, last_seen=?2
             WHERE session_id=?1",
            params![session_id, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        // Keep the legacy turn_state cursor coherent for turn_check_due().
        self.conn.execute(
            "INSERT INTO turn_state (session_id, working, turn_started_at, last_check_at)
             VALUES (?1, 1, ?2, 0)
             ON CONFLICT(session_id) DO UPDATE SET working=1, turn_started_at=?2, last_check_at=0",
            params![session_id, ts],
        )?;
        self.enqueue_status_outbox_current(session_id, ts)?;
        self.local_session_snapshot(session_id)
    }

    /// Place a provisional title IFF none is set yet (title_source='none') AND
    /// `turn_id` still matches the current turn (so a stale seed can't apply).
    /// Returns the updated snapshot when it seeded, else `None`.
    pub fn seed_title_if_empty(
        &self,
        session_id: &str,
        turn_id: i64,
        title: &str,
        ts: u64,
    ) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET
               title=?3, title_source='seed',
               state_version=state_version+1, updated_at=?4, last_seen=?4
             WHERE session_id=?1 AND turn_id=?2 AND title_source='none'",
            params![session_id, turn_id, title, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        self.enqueue_status_outbox_current(session_id, ts)?;
        self.local_session_snapshot(session_id)
    }

    /// Apply a distilled (title, activity). Returns `None` (rejected) unless the
    /// session's CURRENT `(turn_id, state_version)` still equals
    /// `(base_turn_id, base_version)` — so a stale distill or a duplicate runtime
    /// structurally cannot flip the title.
    pub fn apply_distill_result(
        &self,
        session_id: &str,
        base_turn_id: i64,
        base_version: i64,
        title: &str,
        activity: &str,
        ts: u64,
    ) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET
               title=?4, title_source='distill', activity=?5, last_distill_at=?6,
               state_version=state_version+1, updated_at=?6, last_seen=?6
             WHERE session_id=?1 AND turn_id=?2 AND state_version=?3",
            params![session_id, base_turn_id, base_version, title, activity, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        self.enqueue_status_outbox_current(session_id, ts)?;
        self.local_session_snapshot(session_id)
    }

    /// Liveness refresh ONLY: bumps `last_seen`, never `state_version`/`updated_at`,
    /// never enqueues. The daemon re-arms the relay expiration by republishing the
    /// returned snapshot. Returns `None` if the session is unknown.
    pub fn heartbeat_session(&self, session_id: &str, ts: u64) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET last_seen=?2 WHERE session_id=?1",
            params![session_id, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        self.local_session_snapshot(session_id)
    }

    /// Close the turn: idle, live activity cleared, TITLE retained. Resets the
    /// debounce cursor. Bumps version + enqueues (busy changed).
    pub fn end_turn(&self, session_id: &str, ts: u64) -> Result<Option<SessionSnapshot>> {
        let Some(before) = self.local_session_snapshot(session_id)? else {
            return Ok(None);
        };

        if before.busy || !before.activity.is_empty() || before.phase != "idle" {
            self.conn.execute(
                "UPDATE session_state SET
                   busy=0, phase='idle', activity='',
                   state_version=state_version+1, updated_at=?2, last_seen=?2
                 WHERE session_id=?1",
                params![session_id, ts],
            )?;
            self.enqueue_status_outbox_current(session_id, ts)?;
        } else {
            self.conn.execute(
                "UPDATE session_state SET last_seen=?2 WHERE session_id=?1",
                params![session_id, ts],
            )?;
        }
        self.conn.execute(
            "INSERT INTO turn_state (session_id, working, turn_started_at, last_check_at)
             VALUES (?1, 0, 0, 0)
             ON CONFLICT(session_id) DO UPDATE SET working=0, last_check_at=0",
            params![session_id],
        )?;
        self.local_session_snapshot(session_id)
    }

    /// Finish the session: lifecycle='ended', idle, TITLE retained. The final
    /// publish still carries a fresh expiration; beats stop, so it ages off the
    /// relay after STATUS_TTL_SECS (no tombstone). Bumps version + enqueues.
    pub fn end_session(&self, session_id: &str, ts: u64) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET
               busy=0, activity='', phase='idle', lifecycle='ended',
               state_version=state_version+1, updated_at=?2
             WHERE session_id=?1",
            params![session_id, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        self.enqueue_status_outbox_current(session_id, ts)?;
        self.local_session_snapshot(session_id)
    }

    /// Retire a session a newer one replaced (lifecycle='superseded', idle).
    /// Bumps version + enqueues. Called internally by the registry's Supersede
    /// branch and exposed for the daemon's stale-sibling sweep.
    pub fn supersede_session(&self, session_id: &str, ts: u64) -> Result<Option<SessionSnapshot>> {
        let n = self.conn.execute(
            "UPDATE session_state SET
               busy=0, activity='', phase='idle', lifecycle='superseded',
               state_version=state_version+1, updated_at=?2
             WHERE session_id=?1",
            params![session_id, ts],
        )?;
        if n == 0 {
            return Ok(None);
        }
        self.enqueue_status_outbox_current(session_id, ts)?;
        self.local_session_snapshot(session_id)
    }

    // ── canonical session aggregate: read facade ──────────────────────────────

    /// The full snapshot of one local canonical session (any lifecycle).
    pub fn local_session_snapshot(&self, session_id: &str) -> Result<Option<SessionSnapshot>> {
        let sql = format!("SELECT {SESSION_STATE_COLS} FROM session_state WHERE session_id=?1");
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![session_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_session_state(row)?)),
            None => Ok(None),
        }
    }

    /// Local sessions whose heartbeat is fresh (`last_seen>=since`) and lifecycle
    /// is active. `project=None` = all projects. `since=0` = include all.
    pub fn live_session_snapshots(
        &self,
        project: Option<&str>,
        since: u64,
    ) -> Result<Vec<SessionSnapshot>> {
        let sql = format!(
            "SELECT {SESSION_STATE_COLS} FROM session_state
             WHERE lifecycle='active' AND last_seen>=?1 AND (?2 IS NULL OR project=?2)
             ORDER BY last_seen DESC"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![since, project], row_to_session_state)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Relay-confirmed published presence seen at or after `since`.
    /// `project=None` = all. Historically this was backed by
    /// `peer_session_state`; it now reads the cohesive `presence_state`
    /// projection so the source distinction stays in data, not schema shape.
    pub fn peer_session_snapshots(
        &self,
        project: Option<&str>,
        since: u64,
    ) -> Result<Vec<SessionSnapshot>> {
        self.presence_snapshots(project, since)
    }

    /// The shared delta query backing turn-start (subsequent turns) and the
    /// PostToolUse turn_check. Returns appeared / changed / finished-or-left
    /// transitions across BOTH `session_state` and `peer_session_state`, scoped
    /// to `project`, since cursor `since`, self-excluded by `exclude`. `now`
    /// drives liveness/expiry classification.
    ///
    /// A row surfaces when it appeared (`first_seen>=since`), changed
    /// (`updated_at>=since`, i.e. a versioned content change — agent went idle, a
    /// new title), or went gone (lifecycle ended/superseded since `since`, OR its
    /// liveness expired within the window). Pure-read: writes nothing.
    pub fn status_delta_since(
        &self,
        project: &str,
        since: u64,
        now: u64,
        exclude: Option<&str>,
    ) -> Result<Vec<StatusDeltaItem>> {
        let ttl = crate::domain::STATUS_TTL_SECS;
        // Window predicate (same on both tables): appeared OR changed OR
        // expired-within-window.
        let mut out: Vec<StatusDeltaItem> = Vec::new();

        let local_sql = format!(
            "SELECT {SESSION_STATE_COLS} FROM session_state
             WHERE project=?1
               AND (first_seen>=?2 OR updated_at>=?2 OR (last_seen < ?3 AND last_seen+?4 >= ?2))"
        );
        let now_minus_ttl = now.saturating_sub(ttl);
        // Self-echo dedup must cover EVERY local session's pubkey in this
        // project, not only those inside the delta window: a local session that
        // hasn't changed since the cursor still round-trips a fresh kind:30315
        // into peer_session_state, and that echo must never surface to ourselves.
        // Keyed on agent_pubkey since peer rows no longer carry a session_id (#5 §4).
        let mut local_pubkeys: std::collections::HashSet<String> = std::collections::HashSet::new();
        {
            let mut stmt = self
                .conn
                .prepare("SELECT agent_pubkey FROM session_state WHERE project=?1")?;
            let rows = stmt.query_map(params![project], |r| r.get::<_, String>(0))?;
            for pk in rows.filter_map(|r| r.ok()) {
                local_pubkeys.insert(pk);
            }
        }
        {
            let mut stmt = self.conn.prepare(&local_sql)?;
            let rows = stmt.query_map(
                params![project, since, now_minus_ttl, ttl],
                row_to_session_state,
            )?;
            for snap in rows.filter_map(|r| r.ok()) {
                if exclude == Some(snap.session_id.as_str()) {
                    continue;
                }
                if let Some(item) = classify_delta(snap, since, now) {
                    out.push(item);
                }
            }
        }

        for snap in self.presence_delta_snapshots(project, since, now_minus_ttl, ttl)? {
            // exclude is a local session_id (te-*); relay presence usually uses
            // pubkey as session_id unless it is our own confirmed publish.
            if exclude == Some(snap.session_id.as_str()) {
                continue;
            }
            // Dedup: skip relay echoes of our own sessions (keyed by pubkey).
            if local_pubkeys.contains(&snap.agent_pubkey) {
                continue;
            }
            if let Some(item) = classify_delta(snap, since, now) {
                out.push(item);
            }
        }
        Ok(out)
    }

    /// Status deltas across a SET of channels (the current channel ∪ its
    /// subtree). Each channel is classified independently by `status_delta_since`
    /// — so per-channel self-echo dedup is preserved — and the results unioned.
    /// Items carry their `snapshot.project`, so the caller can tag cross-channel
    /// deltas with the originating subchannel.
    pub fn status_delta_since_in(
        &self,
        channels: &[String],
        since: u64,
        now: u64,
        exclude: Option<&str>,
    ) -> Result<Vec<StatusDeltaItem>> {
        let mut out: Vec<StatusDeltaItem> = Vec::new();
        for ch in channels {
            out.extend(self.status_delta_since(ch, since, now, exclude)?);
        }
        Ok(out)
    }

    // ── peer mirror write (kind:30315 materializer surface) ───────────────────

    /// Mirror an inbound kind:30315 into `presence_state`. Idempotent upsert
    /// keyed by `(pubkey, project)` — one row per actor per group; a newer
    /// heartbeat from the same agent replaces the older one. Bumps `state_version`
    /// + `updated_at` only when public content changed (title/activity/busy/host/
    ///   rel_cwd/slug); advances `last_seen` only on a newer `emitted_at` so
    ///   out-of-order refetches never resurrect a finished actor. `first_seen` is
    ///   set once on insert.
    pub fn record_peer_status(&self, obs: &PeerStatusObservation) -> Result<()> {
        self.record_relay_presence(obs, None)
    }
}
