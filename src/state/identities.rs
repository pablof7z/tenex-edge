//! `identities` — derived signing keys the daemon publishes as.
//!
//! (base agent pubkey, ordinal) plus per-session pubkeys map to their owning
//! agent/session and a resume binding. Bounds the `#p` subscription (the set of
//! pubkeys the daemon listens for) and resumes the right session when a mention
//! arrives for an offline agent. Ordinal 0 == the base agent key.

use super::*;

const COLS: &str = "pubkey, base_pubkey, agent_slug, ordinal, session_id, channel_h, native_id, \
     alive, created_at";

fn row_to_identity(row: &rusqlite::Row) -> rusqlite::Result<Identity> {
    Ok(Identity {
        pubkey: row.get(0)?,
        base_pubkey: row.get(1)?,
        agent_slug: row.get(2)?,
        ordinal: row.get::<_, i64>(3)? as u32,
        session_id: row.get(4)?,
        channel_h: row.get(5)?,
        native_id: row.get(6)?,
        alive: row.get::<_, i64>(7)? != 0,
        created_at: row.get(8)?,
    })
}

impl Store {
    /// Upsert a derived identity keyed by its pubkey.
    pub fn upsert_identity(&self, i: &Identity) -> Result<()> {
        self.conn.execute(
            "INSERT INTO identities
                 (pubkey, base_pubkey, agent_slug, ordinal, session_id, channel_h, native_id,
                  alive, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(pubkey) DO UPDATE SET
                 base_pubkey=excluded.base_pubkey, agent_slug=excluded.agent_slug,
                 ordinal=excluded.ordinal, session_id=excluded.session_id,
                 channel_h=excluded.channel_h, native_id=excluded.native_id,
                 alive=excluded.alive",
            params![
                i.pubkey,
                i.base_pubkey,
                i.agent_slug,
                i.ordinal as i64,
                i.session_id,
                i.channel_h,
                i.native_id,
                i.alive as i64,
                i.created_at
            ],
        )?;
        Ok(())
    }

    /// Fetch one identity by its (derived) pubkey.
    pub fn get_identity(&self, pubkey: &str) -> Result<Option<Identity>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM identities WHERE pubkey=?1"),
                params![pubkey],
                row_to_identity,
            )
            .optional()?)
    }

    /// Every derived pubkey the daemon signs as — the bound on the `#p`
    /// subscription.
    pub fn list_identity_pubkeys(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT pubkey FROM identities ORDER BY base_pubkey, ordinal")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// All identities sharing a base agent key (ordinal 0..n).
    pub fn identities_for_base(&self, base_pubkey: &str) -> Result<Vec<Identity>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM identities WHERE base_pubkey=?1 ORDER BY ordinal"
        ))?;
        let rows = stmt.query_map(params![base_pubkey], row_to_identity)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Bind a derived identity to a live harness session for resume: records the
    /// canonical session id (resolved first), the harness-native id, and liveness.
    pub fn bind_session_identity(
        &self,
        pubkey: &str,
        session_id: &str,
        native_id: &str,
        alive: bool,
    ) -> Result<()> {
        let canonical = self
            .resolve_canonical_id(session_id)?
            .unwrap_or_else(|| session_id.to_string());
        self.conn.execute(
            "UPDATE identities SET session_id=?2, native_id=?3, alive=?4 WHERE pubkey=?1",
            params![pubkey, canonical, native_id, alive as i64],
        )?;
        Ok(())
    }

    /// The identity currently bound to a canonical session (resolves the id
    /// first). Used by `who`/signing/reconcile to report a session's durable
    /// ordinal identity. Newest binding wins.
    pub fn identity_for_session(&self, session_id: &str) -> Result<Option<Identity>> {
        let Some(canonical) = self.resolve_canonical_id(session_id)? else {
            return Ok(None);
        };
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLS} FROM identities WHERE session_id=?1
                     ORDER BY created_at DESC LIMIT 1"
                ),
                params![canonical],
                row_to_identity,
            )
            .optional()?)
    }

    /// The authoritative [`crate::identity::AgentInstance`] for a session (issue
    /// #98): the projection of its bound `identities` row that read-side callers
    /// (status/chat publish, `who`/statusline, mention routing) consume instead of
    /// re-deriving base-vs-ordinal label/pubkey/key policy at the edge. `None` when
    /// no derived identity is bound yet (callers fall back to the base instance
    /// from the session row).
    pub fn instance_identity_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<crate::identity::AgentInstance>> {
        Ok(self.identity_for_session(session_id)?.map(|i| {
            crate::identity::AgentInstance::from_parts(
                i.agent_slug,
                i.base_pubkey,
                i.ordinal,
                i.pubkey,
            )
        }))
    }

    /// Mark every identity bound to a session dead (alive=0) while KEEPING the row
    /// so a later mention can resume its bound native session. Resolves the id.
    pub fn mark_identity_dead_for_session(&self, session_id: &str) -> Result<()> {
        let Some(canonical) = self.resolve_canonical_id(session_id)? else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE identities SET alive=0 WHERE session_id=?1",
            params![canonical],
        )?;
        Ok(())
    }

    /// The identity bound to a base agent in a given channel — used to resume the
    /// right session when a mention arrives for an offline agent. Prefers a live
    /// binding, then the most recent.
    pub fn resolve_identity_for_channel(
        &self,
        base_pubkey: &str,
        channel_h: &str,
    ) -> Result<Option<Identity>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLS} FROM identities
                     WHERE base_pubkey=?1 AND channel_h=?2
                     ORDER BY alive DESC, created_at DESC LIMIT 1"
                ),
                params![base_pubkey, channel_h],
                row_to_identity,
            )
            .optional()?)
    }
}
