use super::*;

impl Store {
    pub fn upsert_profile(
        &self,
        pubkey: &str,
        slug: &str,
        host: &str,
        is_backend: bool,
        ts: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO profiles (pubkey, slug, host, is_backend, updated_at) VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(pubkey) DO UPDATE SET slug=?2, host=?3, is_backend=?4, updated_at=?5",
            params![pubkey, slug, host, is_backend as i64, ts],
        )?;
        Ok(())
    }

    /// Returns `true` if the stored profile for `pubkey` is a tenex-edge backend
    /// (published with a `["backend"]` tag). Used to suppress backends from the
    /// agent-facing channel member context.
    pub fn is_backend_profile(&self, pubkey: &str) -> bool {
        self.conn
            .query_row(
                "SELECT is_backend FROM profiles WHERE pubkey=?1 LIMIT 1",
                params![pubkey],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v != 0)
            .unwrap_or(false)
    }

    /// The cached kind:0 display name for `pubkey` and when it was last written,
    /// straight from the `profiles` table. Returns `None` when no profile is
    /// cached. The caller (the kind:0 resolver) uses `updated_at` to decide
    /// whether the entry is fresh enough or must be re-fetched from a relay.
    pub fn cached_profile(&self, pubkey: &str) -> Option<(String, u64)> {
        self.conn
            .query_row(
                "SELECT slug, updated_at FROM profiles WHERE pubkey=?1 LIMIT 1",
                params![pubkey],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?)),
            )
            .ok()
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
    pub fn resolve_agent_pubkey(
        &self,
        slug: &str,
        project: Option<&str>,
    ) -> Result<Option<String>> {
        if let Some(project) = project {
            return Ok(self
                .conn
                .query_row(
                    "SELECT pubkey FROM peer_sessions WHERE slug=?1 AND project=?2 ORDER BY last_seen DESC LIMIT 1",
                    params![slug, project],
                    |r| r.get::<_, String>(0),
                )
                .ok());
        }

        if let Ok(pk) = self.conn.query_row(
            "SELECT pubkey FROM profiles WHERE slug=?1 ORDER BY updated_at DESC LIMIT 1",
            params![slug],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(pk));
        }
        Ok(self
            .conn
            .query_row(
                "SELECT pubkey FROM peer_sessions WHERE slug=?1 ORDER BY last_seen DESC LIMIT 1",
                params![slug],
                |r| r.get::<_, String>(0),
            )
            .ok())
    }

    /// Canonical `agent@host` lookup: resolve a durable agent on a specific
    /// machine — `(slug, slugify_host(host))` → pubkey. Scans kind:0 profiles
    /// (covers remote agents) then own sessions, filtering by the slugified host.
    /// This is the one place `@host` addressing resolves; `@` never means project.
    pub fn pubkey_for_agent_on_host(&self, slug: &str, host_slug: &str) -> Result<Option<String>> {
        let scan = |sql: &str| -> Option<String> {
            let mut stmt = self.conn.prepare(sql).ok()?;
            let rows = stmt
                .query_map(params![slug], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .ok()?;
            for (pubkey, host) in rows.flatten() {
                if crate::util::slugify_host(&host) == host_slug {
                    return Some(pubkey);
                }
            }
            None
        };
        if let Some(pk) =
            scan("SELECT pubkey, host FROM profiles WHERE slug=?1 ORDER BY updated_at DESC")
        {
            return Ok(Some(pk));
        }
        if let Some(pk) = scan(
            "SELECT agent_pubkey, host FROM sessions WHERE agent_slug=?1 ORDER BY created_at DESC",
        ) {
            return Ok(Some(pk));
        }
        Ok(None)
    }

    /// Resolve a backend pubkey from a host slug (the `slugify_host` form shown
    /// by `who`). Queries peer_sessions then profiles for any row whose host,
    /// when slugified, equals `host_slug`. Returns the most-recently-seen pubkey,
    /// or `None` if no peer is known under that host name.
    pub fn pubkey_for_host_slug(&self, host_slug: &str) -> Option<String> {
        use crate::util::slugify_host;
        // peer_sessions: prefer the most-recently active backend on that host.
        {
            let mut stmt = self
                .conn
                .prepare("SELECT pubkey, host FROM peer_sessions ORDER BY last_seen DESC")
                .ok()?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()?;
            for row in rows.flatten() {
                if slugify_host(&row.1) == host_slug {
                    return Some(row.0);
                }
            }
        }
        // profiles: fall back to kind:0 identity cards.
        {
            let mut stmt = self
                .conn
                .prepare("SELECT pubkey, host FROM profiles ORDER BY updated_at DESC")
                .ok()?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()?;
            for row in rows.flatten() {
                if slugify_host(&row.1) == host_slug {
                    return Some(row.0);
                }
            }
        }
        None
    }

    /// Reverse-lookup: given a pubkey, return the slug this agent is known by
    /// (from own sessions, peer_sessions, or profiles). Returns None if completely unknown.
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

    pub fn resolve_slug_for_pubkey(&self, pubkey: &str) -> Result<Option<String>> {
        // Check own sessions first (most authoritative for local agents).
        if let Ok(slug) = self.conn.query_row(
            "SELECT agent_slug FROM sessions WHERE agent_pubkey=?1 ORDER BY created_at DESC LIMIT 1",
            params![pubkey],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(slug));
        }
        // Then peer_sessions (remote agents seen recently).
        if let Ok(slug) = self.conn.query_row(
            "SELECT slug FROM peer_sessions WHERE pubkey=?1 ORDER BY last_seen DESC LIMIT 1",
            params![pubkey],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(slug));
        }
        // Fall back to profiles table (populated by kind:0 events from peers).
        if let Ok(slug) = self.conn.query_row(
            "SELECT slug FROM profiles WHERE pubkey=?1 LIMIT 1",
            params![pubkey],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(slug));
        }
        // Stage 3: check if the pubkey is a per-session derived key. Local
        // sessions skip profile materialization (is_self gate), so the profiles
        // table won't have an entry. Fabricate "<codename> (<agent_slug>)"
        // matching the session kind:0 we publish with the session key.
        if let Some((session_id, _agent_pubkey, agent_slug)) = self.session_pubkey_info(pubkey) {
            let codename = crate::util::session_codename(&session_id);
            return Ok(Some(format!("{codename} ({agent_slug})")));
        }
        Ok(None)
    }

    pub fn resolve_chat_host(
        &self,
        pubkey: &str,
        from_session: Option<&str>,
    ) -> Result<Option<String>> {
        if let Some(session_id) = from_session.filter(|s| !s.is_empty()) {
            if let Ok(host) = self.conn.query_row(
                "SELECT host FROM sessions WHERE session_id=?1 LIMIT 1",
                params![session_id],
                |r| r.get::<_, String>(0),
            ) {
                return Ok(Some(host));
            }
            if let Ok(host) = self.conn.query_row(
                "SELECT host FROM peer_sessions WHERE session_id=?1 LIMIT 1",
                params![session_id],
                |r| r.get::<_, String>(0),
            ) {
                return Ok(Some(host));
            }
        }
        if let Ok(host) = self.conn.query_row(
            "SELECT host FROM sessions WHERE agent_pubkey=?1 ORDER BY created_at DESC LIMIT 1",
            params![pubkey],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(host));
        }
        if let Ok(host) = self.conn.query_row(
            "SELECT host FROM peer_sessions WHERE pubkey=?1 ORDER BY last_seen DESC LIMIT 1",
            params![pubkey],
            |r| r.get::<_, String>(0),
        ) {
            return Ok(Some(host));
        }
        Ok(self
            .conn
            .query_row(
                "SELECT host FROM profiles WHERE pubkey=?1 LIMIT 1",
                params![pubkey],
                |r| r.get::<_, String>(0),
            )
            .ok())
    }

    /// Find one of MY sessions by session-id prefix (for messaging a sibling
    /// session of the same agent on this machine).
    pub fn find_session_by_prefix(&self, prefix: &str) -> Result<Option<SessionRecord>> {
        let pat = format!("{prefix}%");
        let mut stmt = self.conn.prepare(
            "SELECT session_id, agent_slug, agent_pubkey, project, host, child_pid, watch_pid, created_at, alive, rel_cwd, channel
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

    /// Peer sessions first seen at or after `since`, still live (last_seen >= fresh_since).
    pub fn list_new_peer_sessions(
        &self,
        since: u64,
        fresh_since: u64,
        project: Option<&str>,
    ) -> Result<Vec<PeerSession>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, pubkey, slug, project, host, last_seen, rel_cwd FROM peer_sessions
             WHERE first_seen>=?1 AND last_seen>=?2 AND (?3 IS NULL OR project=?3)
             ORDER BY first_seen",
        )?;
        let rows: Vec<PeerSession> = stmt
            .query_map(params![since, fresh_since, project], row_to_peer)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }
}
