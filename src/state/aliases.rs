//! `session_aliases` — external id -> canonical session (N:1, repointable).
//!
//! Reused OS slots (tmux pane, watch pid) and rotated harness ids repoint to the
//! newest live owner. Keyed by `(harness, external_id_kind, external_id)`.

use super::*;

impl Store {
    /// Point an external id at a canonical session. Re-pointing a reused slot or a
    /// rotated harness id is an ON CONFLICT update to the newest owner.
    pub fn put_alias(
        &self,
        harness: &str,
        external_id_kind: &str,
        external_id: &str,
        session_id: &str,
        created_at: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO session_aliases
                 (harness, external_id_kind, external_id, session_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(harness, external_id_kind, external_id)
                 DO UPDATE SET session_id=excluded.session_id, created_at=excluded.created_at",
            params![harness, external_id_kind, external_id, session_id, created_at],
        )?;
        Ok(())
    }

    /// Resolve a specific external id (by its full key) to its canonical session.
    pub fn resolve_session_by_alias(
        &self,
        harness: &str,
        external_id_kind: &str,
        external_id: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT session_id FROM session_aliases
                 WHERE harness=?1 AND external_id_kind=?2 AND external_id=?3",
                params![harness, external_id_kind, external_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?)
    }

    /// All aliases of a given kind across every session, newest first (e.g. all
    /// `tmux_pane` bindings to enumerate live tmux endpoints).
    pub fn list_aliases_of_kind(&self, external_id_kind: &str) -> Result<Vec<SessionAlias>> {
        let mut stmt = self.conn.prepare(
            "SELECT harness, external_id_kind, external_id, session_id, created_at
             FROM session_aliases WHERE external_id_kind=?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![external_id_kind], |row| {
            Ok(SessionAlias {
                harness: row.get(0)?,
                external_id_kind: row.get(1)?,
                external_id: row.get(2)?,
                session_id: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Drop the `tmux_pane` alias(es) for a session (resolves the id first). Used
    /// when the bound pane is found dead, so it is no longer treated as an endpoint.
    pub fn clear_tmux_pane(&self, session_id: &str) -> Result<()> {
        let target = self
            .resolve_canonical_id(session_id)?
            .unwrap_or_else(|| session_id.to_string());
        self.conn.execute(
            "DELETE FROM session_aliases WHERE session_id=?1 AND external_id_kind='tmux_pane'",
            params![target],
        )?;
        Ok(())
    }

    /// All external-id aliases pointing at a canonical session (e.g. to retire
    /// them when the session dies).
    pub fn aliases_for_session(&self, session_id: &str) -> Result<Vec<SessionAlias>> {
        let mut stmt = self.conn.prepare(
            "SELECT harness, external_id_kind, external_id, session_id, created_at
             FROM session_aliases WHERE session_id=?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(SessionAlias {
                harness: row.get(0)?,
                external_id_kind: row.get(1)?,
                external_id: row.get(2)?,
                session_id: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
