use super::*;

impl Store {
    pub fn list_projects_read_model(&self) -> Result<Vec<(String, String)>> {
        self.list_project_meta()
    }

    /// About-text for a single project by its legacy slug.
    ///
    // Retained storage (Phase 8): project_meta is the deliberately-retained canonical home for
    // project slug+about; readers query it directly per fabric-architecture.md §6.
    pub fn project_meta_read_model(&self, slug: &str) -> Result<Option<String>> {
        self.get_project_meta(slug)
    }

    /// Own (local) sessions that are still alive and recently heartbeated.
    ///
    // Retained storage (Phase 8): sessions is the deliberately-retained canonical home for
    // local agent sessions; readers query it directly per fabric-architecture.md §6.
    pub fn list_agents_read_model(
        &self,
        project: Option<&str>,
        since: u64,
    ) -> Result<Vec<SessionRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
             FROM sessions WHERE alive=1 AND last_seen>=?1 AND (?2 IS NULL OR project=?2) ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![since, project], row_to_session)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Peer presence rows, ordered by recency.
    ///
    // Retained storage (Phase 8): peer_sessions is the deliberately-retained canonical home for
    // peer presence; readers query it directly per fabric-architecture.md §6.
    pub fn list_presence_read_model(
        &self,
        project: Option<&str>,
        since: u64,
    ) -> Result<Vec<PeerSession>> {
        self.list_peer_sessions(project, since)
    }

    /// Resolve a project display-slug to its canonical `project_id` for the
    /// NIP-29 fabric. Read-only — does NOT create an origin.
    pub fn project_id_for_slug(
        &self,
        fabric: &str,
        provider_instance: &str,
        slug: &str,
    ) -> Result<Option<String>> {
        self.project_id_for_origin(fabric, provider_instance, slug)
    }

    /// Explicit chat mentions already drained to `session_id` at or after `since`.
    pub fn list_recently_delivered_chat_mentions(
        &self,
        session_id: &str,
        since: u64,
    ) -> Result<Vec<ChatInboxRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chat_event_id, target_session, from_pubkey, from_slug, project, body, created_at, from_session, mentioned_session
             FROM chat_inbox
             WHERE target_session=?1 AND mentioned_session=?1 AND delivered=1 AND delivered_at>=?2
             ORDER BY created_at",
        )?;
        let rows: Vec<ChatInboxRow> = stmt
            .query_map(params![session_id, since], row_to_chat)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    // ── Phase 2: write-facing materializer methods ───────────────────────────
    //
    // These are the write surface the Phase 4 materializer will call.  Nothing
    // calls them in Phase 2; they exist so the seam compiles, so the signatures
    // are locked, and so unit tests prevent dead-code warnings.

    /// Upsert a peer profile (kind:0).  Wraps `upsert_profile`.
    pub fn materialize_profile(&self, pubkey: &str, slug: &str, host: &str, ts: u64) -> Result<()> {
        self.upsert_profile(pubkey, slug, host, false, ts)
    }

    /// Record / refresh a peer presence session (kind:0 + relay presence).
    /// Wraps `upsert_peer_session`.
    #[allow(clippy::too_many_arguments)] // mirrors upsert_peer_session's column set
    pub fn materialize_presence(
        &self,
        session_id: &str,
        pubkey: &str,
        slug: &str,
        project: &str,
        host: &str,
        rel_cwd: &str,
        ts: u64,
    ) -> Result<()> {
        self.upsert_peer_session(session_id, pubkey, slug, project, host, rel_cwd, ts)
    }

    /// Apply a relay-authoritative NIP-29 39002 membership snapshot:
    /// replaces the legacy `group_members` cache wholesale AND mirrors into
    /// canonical `membership` rows via `admit_member` (source `"nip29-39002"`).
    ///
    /// `provider_instance` is the relay-set hash (daemon-derived); used to
    /// resolve the canonical `project_id` via `project_id_for_origin`.
    pub fn materialize_membership_snapshot(
        &self,
        project_slug: &str,
        members: &[(String, String)],
        provider_instance: &str,
        ts: u64,
    ) -> Result<()> {
        // Legacy table: authoritative wholesale replace.
        self.replace_group_members(project_slug, members, ts)?;
        // Canonical mirror via Phase 1 accessor.
        const FABRIC: &str = "nip29";
        if let Some(pid) = self.project_id_for_origin(FABRIC, provider_instance, project_slug)? {
            for (pubkey, role) in members {
                self.admit_member(&pid, pubkey, role, "nip29-39002", ts)?;
            }
        }
        Ok(())
    }

    /// Record a distillation failure for this session (upserts — only the last
    /// error is kept in the DB; the log file retains full history).
    pub fn record_session_error(&self, session_id: &str, message: &str, ts: u64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO session_errors (session_id, message, ts) VALUES (?1, ?2, ?3)",
            rusqlite::params![session_id, message, ts],
        )?;
        Ok(())
    }

    /// Return the last distillation error for `session_id` if it occurred at or
    /// after `since` (unix seconds). Returns `None` when no recent error exists.
    pub fn get_recent_session_error(&self, session_id: &str, since: u64) -> Result<Option<String>> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT message FROM session_errors WHERE session_id = ?1 AND ts >= ?2",
            rusqlite::params![session_id, since],
            |row| row.get(0),
        );
        match result {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
