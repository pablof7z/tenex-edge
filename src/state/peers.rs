use super::*;

impl Store {
    // ── peer directory ───────────────────────────────────────────────────

    pub fn slug_for_pubkey(&self, pubkey: &str) -> String {
        self.conn
            .query_row(
                "SELECT slug FROM profiles WHERE pubkey = ?1",
                params![pubkey],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default()
    }

    pub fn upsert_profile(&self, pubkey: &str, slug: &str, host: &str, ts: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO profiles (pubkey, slug, host, updated_at) VALUES (?1,?2,?3,?4)
             ON CONFLICT(pubkey) DO UPDATE SET slug=?2, host=?3, updated_at=?4",
            params![pubkey, slug, host, ts],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_peer_session(
        &self,
        session_id: &str,
        pubkey: &str,
        slug: &str,
        project: &str,
        host: &str,
        rel_cwd: &str,
        ts: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO peer_sessions (session_id, pubkey, slug, project, host, rel_cwd, last_seen, first_seen)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?7)
             ON CONFLICT(session_id) DO UPDATE SET pubkey=?2, slug=?3, project=?4, host=?5, rel_cwd=?6, last_seen=?7",
            params![session_id, pubkey, slug, project, host, rel_cwd, ts],
        )?;
        Ok(())
    }

    /// Resolve an agent slug to a pubkey. With a project scope, this behaves
    /// like `slug@project`: prefer presence in that project, and do not let a
    /// global profile from another project hijack the route.
    ///
    /// Falls back to the local `sessions` table (including `alive=0` rows) so
    /// that own agents without active relay presence can still be messaged.
    pub fn resolve_agent_pubkey(
        &self,
        slug: &str,
        project: Option<&str>,
    ) -> Result<Option<String>> {
        if let Some(project) = project {
            // Try relay-announced peer sessions first.
            if let Ok(pk) = self
                .conn
                .query_row(
                    "SELECT pubkey FROM peer_sessions WHERE slug=?1 AND project=?2 ORDER BY last_seen DESC LIMIT 1",
                    params![slug, project],
                    |r| r.get::<_, String>(0),
                )
            {
                return Ok(Some(pk));
            }
            // Fall back to local sessions table (own agents, even dead sessions).
            return Ok(self
                .conn
                .query_row(
                    "SELECT agent_pubkey FROM sessions WHERE agent_slug=?1 AND project=?2 ORDER BY created_at DESC LIMIT 1",
                    params![slug, project],
                    |r| r.get::<_, String>(0),
                )
                .ok());
        }

        if let Ok(pk) = self
            .conn
            .query_row(
                "SELECT pubkey FROM profiles WHERE slug=?1 ORDER BY updated_at DESC LIMIT 1",
                params![slug],
                |r| r.get::<_, String>(0),
            )
        {
            return Ok(Some(pk));
        }
        if let Ok(pk) = self
            .conn
            .query_row(
                "SELECT pubkey FROM peer_sessions WHERE slug=?1 ORDER BY last_seen DESC LIMIT 1",
                params![slug],
                |r| r.get::<_, String>(0),
            )
        {
            return Ok(Some(pk));
        }
        // Fall back to local sessions table (own agents, even dead sessions).
        Ok(self
            .conn
            .query_row(
                "SELECT agent_pubkey FROM sessions WHERE agent_slug=?1 ORDER BY created_at DESC LIMIT 1",
                params![slug],
                |r| r.get::<_, String>(0),
            )
            .ok())
    }

    /// Look up the agent slug for a locally-owned pubkey from the `sessions`
    /// table (including `alive=0` rows). Returns `None` for remote-only pubkeys
    /// that have no local session record — callers use this as the "is locally
    /// owned?" gate before attempting a tmux spawn.
    pub fn get_local_agent_slug_by_pubkey(&self, pubkey: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT agent_slug FROM sessions WHERE agent_pubkey=?1 ORDER BY created_at DESC LIMIT 1",
                params![pubkey],
                |r| r.get::<_, String>(0),
            )
            .ok()
    }

    /// Find one of MY sessions by session-id prefix (for messaging a sibling
    /// session of the same agent on this machine).
    pub fn find_session_by_prefix(&self, prefix: &str) -> Result<Option<SessionRecord>> {
        let pat = format!("{prefix}%");
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd
             FROM sessions WHERE session_id LIKE ?1 ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![pat])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_session(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_peer_session_by_prefix(&self, prefix: &str) -> Result<Option<PeerSession>> {
        let pat = format!("{prefix}%");
        let mut stmt = self.conn.prepare(
            "SELECT session_id, pubkey, slug, project, host, last_seen, rel_cwd
             FROM peer_sessions WHERE session_id LIKE ?1 ORDER BY last_seen DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![pat])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_peer(row)?))
        } else {
            Ok(None)
        }
    }

    /// Peer sessions seen at or after `since` (freshness filter). `project=None`
    /// = all projects. A peer is "live" only while its heartbeat keeps `last_seen`
    /// fresh; once heartbeats stop it ages out and is no longer shown.
    pub fn list_peer_sessions(
        &self,
        project: Option<&str>,
        since: u64,
    ) -> Result<Vec<PeerSession>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, pubkey, slug, project, host, last_seen, rel_cwd FROM peer_sessions
             WHERE last_seen>=?1 AND (?2 IS NULL OR project=?2) ORDER BY last_seen DESC",
        )?;
        let rows: Vec<PeerSession> = stmt
            .query_map(params![since, project], row_to_peer)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    /// Delete peer sessions not seen since `before` (housekeeping for pollution).
    pub fn prune_peer_sessions(&self, before: u64) -> Result<usize> {
        Ok(self.conn.execute(
            "DELETE FROM peer_sessions WHERE last_seen<?1",
            params![before],
        )?)
    }

    // ── ACL: pending agents (kind:0 claiming us, not yet authorized) ──────

    pub fn upsert_pending_agent(
        &self,
        pubkey: &str,
        slug: &str,
        host: &str,
        owners: &str,
        ts: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO pending_agents (pubkey, slug, host, owners, first_seen) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(pubkey) DO UPDATE SET slug=?2, host=?3, owners=?4",
            params![pubkey, slug, host, owners, ts],
        )?;
        Ok(())
    }

    pub fn remove_pending_agent(&self, pubkey: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM pending_agents WHERE pubkey=?1",
            params![pubkey],
        )?;
        Ok(())
    }

    pub fn list_pending_agents(&self) -> Result<Vec<PendingAgent>> {
        let mut stmt = self.conn.prepare(
            "SELECT pubkey, slug, host, owners, first_seen FROM pending_agents ORDER BY first_seen",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PendingAgent {
                    pubkey: row.get(0)?,
                    slug: row.get(1)?,
                    host: row.get(2)?,
                    owners: row.get(3)?,
                    first_seen: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }
}
