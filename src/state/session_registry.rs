use super::*;

impl Store {
    pub fn register_or_reassert_session(
        &self,
        obs: &SessionObservation,
    ) -> Result<SessionSnapshot> {
        let alias_hit = self.alias_lookup(obs);
        let live = self.live_locators_for(&obs.host, &obs.project, &obs.agent_pubkey, obs)?;
        let decision = crate::session::resolve_identity(obs, alias_hit, &live);
        let id = match decision {
            IdentityDecision::Existing(id) | IdentityDecision::Reattach(id) => {
                self.reassert_session_row(id.as_str(), obs)?;
                id.into_string()
            }
            IdentityDecision::Supersede { old } => {
                self.supersede_session(old.as_str(), obs.observed_at)?;
                let id = mint_session_id();
                self.insert_session_row(&id, obs)?;
                id
            }
            IdentityDecision::Mint => {
                let id = mint_session_id();
                self.insert_session_row(&id, obs)?;
                id
            }
        };
        self.write_session_aliases(&id, obs)?;
        Ok(self
            .local_session_snapshot(&id)?
            .expect("session_state row written by register_or_reassert_session"))
    }

    /// Existing-id path: refresh mutable identity fields + liveness. Only bump
    /// the version / `updated_at` when the public status actually changed.
    fn reassert_session_row(&self, session_id: &str, obs: &SessionObservation) -> Result<()> {
        let before = self.local_session_snapshot(session_id)?;
        let public_changed = before
            .as_ref()
            .map(|s| {
                s.agent_slug != obs.agent_slug
                    || s.host != obs.host
                    || s.rel_cwd != obs.rel_cwd
                    || !s.lifecycle.is_active()
            })
            .unwrap_or(true);

        if public_changed {
            self.conn.execute(
                "UPDATE session_state SET
                   agent_slug=?2, host=?3, rel_cwd=?4,
                   resume_id=CASE WHEN ?5<>'' THEN ?5 ELSE resume_id END,
                   last_seen=?6, lifecycle='active',
                   state_version=state_version+1, updated_at=?6
                 WHERE session_id=?1",
                params![
                    session_id,
                    obs.agent_slug,
                    obs.host,
                    obs.rel_cwd,
                    obs.resume_id.clone().unwrap_or_default(),
                    obs.observed_at,
                ],
            )?;
            self.enqueue_status_outbox_current(session_id, obs.observed_at)
        } else {
            self.conn.execute(
                "UPDATE session_state SET
                   resume_id=CASE WHEN ?2<>'' THEN ?2 ELSE resume_id END,
                   last_seen=?3, lifecycle='active'
                 WHERE session_id=?1",
                params![
                    session_id,
                    obs.resume_id.clone().unwrap_or_default(),
                    obs.observed_at,
                ],
            )?;
            Ok(())
        }
    }

    /// Mint-path insert: a brand-new canonical row at version 1.
    fn insert_session_row(&self, session_id: &str, obs: &SessionObservation) -> Result<()> {
        self.conn.execute(
            "INSERT INTO session_state
               (session_id, agent_slug, agent_pubkey, project, host, rel_cwd,
                title, title_source, activity, busy, phase, turn_id, turn_started_at,
                last_distill_at, last_seen, resume_id, state_version, lifecycle,
                first_seen, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6, '', 'none', '', 0, 'idle', 0, 0,
                     0, ?7, ?8, 1, 'active', ?7, ?7)",
            params![
                session_id,
                obs.agent_slug,
                obs.agent_pubkey,
                obs.project,
                obs.host,
                obs.rel_cwd,
                obs.observed_at,
                obs.resume_id.clone().unwrap_or_default(),
            ],
        )?;
        // Keep `sessions` in sync: rpc_session_start also calls upsert_session, but
        // the reassert path (user-prompt-submit → register_or_reassert_session) does
        // not, leaving get_session_exact unable to find the canonical row. Also set
        // last_seen so list_my_live_sessions (last_seen>=?) finds the row before the
        // rpc_session_start touch_session arrives.
        self.conn.execute(
            "INSERT INTO sessions
               (session_id, agent_slug, agent_pubkey, project, host,
                child_pid, watch_pid, created_at, alive, rel_cwd, last_seen)
             VALUES (?1,?2,?3,?4,?5, NULL,?6,?7,1,?8,?7)
             ON CONFLICT(session_id) DO NOTHING",
            params![
                session_id,
                obs.agent_slug,
                obs.agent_pubkey,
                obs.project,
                obs.host,
                obs.watch_pid,
                obs.observed_at,
                obs.rel_cwd,
            ],
        )?;
        self.enqueue_status_outbox(session_id, 1, obs.observed_at)
    }

    /// Upsert every external id the observation carries → this canonical id.
    /// pane/pid/harness aliases are repointed to the newest session so a reused
    /// slot resolves to the live owner.
    fn write_session_aliases(&self, session_id: &str, obs: &SessionObservation) -> Result<()> {
        use crate::session::AliasKind::*;
        let h = obs.harness.as_str();
        let put = |kind: &str, val: &str| -> Result<()> {
            if val.is_empty() {
                return Ok(());
            }
            self.conn.execute(
                "INSERT INTO session_aliases (harness, external_id_kind, external_id, session_id, created_at)
                 VALUES (?1,?2,?3,?4,?5)
                 ON CONFLICT(harness, external_id_kind, external_id)
                 DO UPDATE SET session_id=?4, created_at=?5",
                params![h, kind, val, session_id, obs.observed_at],
            )?;
            Ok(())
        };
        if let Some(v) = &obs.harness_session_id {
            put(HarnessSession.as_str(), v)?;
        }
        if let Some(v) = &obs.resume_id {
            put(Resume.as_str(), v)?;
        }
        if let Some(v) = &obs.tmux_pane {
            put(TmuxPane.as_str(), v)?;
        }
        if let Some(pid) = obs.watch_pid {
            put(WatchPid.as_str(), &pid.to_string())?;
        }
        Ok(())
    }

    /// Alias hit (Existing) consults only harness-native id + resume kinds — a
    /// pane/pid alias from a prior occupant must NOT read as the same session.
    /// Returns the canonical id when one is found AND its row still exists.
    fn alias_lookup(&self, obs: &SessionObservation) -> Option<SessionId> {
        use crate::session::AliasKind;
        let h = obs.harness.as_str();
        // Echo harnesses (e.g. opencode) own no native id, so the daemon mints the
        // canonical id at session-start and echoes it back; the harness then reports
        // it as its own `harness_session_id` on every later hook. That id IS the
        // session — recognize it directly so a reassert REATTACHES instead of falling
        // through to the pane/pid supersede branch and minting a fresh session each
        // first turn. Safe for claude/codex: their native ids are never `te-*`
        // canonical ids, so this never matches for them.
        if let Some(v) = &obs.harness_session_id {
            if !v.is_empty() {
                let is_canonical: bool = self
                    .conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM session_state WHERE session_id=?1)",
                        params![v],
                        |r| r.get(0),
                    )
                    .unwrap_or(false);
                if is_canonical {
                    return Some(SessionId::from(v.clone()));
                }
            }
        }
        let mut candidates: Vec<(&str, &str)> = Vec::new();
        if let Some(v) = &obs.harness_session_id {
            candidates.push((AliasKind::HarnessSession.as_str(), v));
        }
        if let Some(v) = &obs.resume_id {
            candidates.push((AliasKind::Resume.as_str(), v));
        }
        for (kind, val) in candidates {
            if val.is_empty() {
                continue;
            }
            let found: Option<String> = self
                .conn
                .query_row(
                    "SELECT a.session_id FROM session_aliases a
                     JOIN session_state s ON s.session_id=a.session_id
                     WHERE a.harness=?1 AND a.external_id_kind=?2 AND a.external_id=?3",
                    params![h, kind, val],
                    |r| r.get::<_, String>(0),
                )
                .ok();
            if let Some(id) = found {
                return Some(SessionId::from(id));
            }
        }
        None
    }

    /// Live (active + fresh) session candidates on the same (host, project,
    /// agent), with their pane/pid/harness/resume locators joined from
    /// `session_aliases` — the input to `resolve_identity`'s supersede branch.
    fn live_locators_for(
        &self,
        host: &str,
        project: &str,
        agent_pubkey: &str,
        obs: &SessionObservation,
    ) -> Result<Vec<LiveLocator>> {
        use crate::session::AliasKind;
        let fresh_since = obs
            .observed_at
            .saturating_sub(crate::domain::STATUS_TTL_SECS);
        let h = obs.harness.as_str();
        let mut stmt = self.conn.prepare(
            "SELECT s.session_id,
               (SELECT external_id FROM session_aliases a WHERE a.session_id=s.session_id AND a.harness=?1 AND a.external_id_kind=?5),
               (SELECT external_id FROM session_aliases a WHERE a.session_id=s.session_id AND a.harness=?1 AND a.external_id_kind=?6),
               (SELECT external_id FROM session_aliases a WHERE a.session_id=s.session_id AND a.harness=?1 AND a.external_id_kind=?7),
               (SELECT external_id FROM session_aliases a WHERE a.session_id=s.session_id AND a.harness=?1 AND a.external_id_kind=?8)
             FROM session_state s
             WHERE s.lifecycle='active' AND s.host=?2 AND s.project=?3 AND s.agent_pubkey=?4
               AND s.last_seen>=?9",
        )?;
        let rows = stmt
            .query_map(
                params![
                    h,
                    host,
                    project,
                    agent_pubkey,
                    AliasKind::HarnessSession.as_str(),
                    AliasKind::Resume.as_str(),
                    AliasKind::TmuxPane.as_str(),
                    AliasKind::WatchPid.as_str(),
                    fresh_since,
                ],
                |r| {
                    Ok(LiveLocator {
                        session_id: SessionId::from(r.get::<_, String>(0)?),
                        harness_session_id: r.get::<_, Option<String>>(1)?,
                        resume_id: r.get::<_, Option<String>>(2)?,
                        tmux_pane: r.get::<_, Option<String>>(3)?,
                        watch_pid: r
                            .get::<_, Option<String>>(4)?
                            .and_then(|s| s.parse::<i32>().ok()),
                    })
                },
            )?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }
}
