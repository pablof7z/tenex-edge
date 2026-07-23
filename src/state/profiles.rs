//! `relay_profiles` — kind:0 metadata cache, keyed by pubkey.

use super::*;

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<Profile> {
    Ok(Profile {
        pubkey: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        agent_slug: row.get(3)?,
        host: row.get(4)?,
        is_backend: row.get::<_, i64>(5)? != 0,
        agents: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
        workspaces: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        updated_at: row.get(8)?,
    })
}

const COLS: &str =
    "pubkey, name, slug, agent_slug, host, is_backend, agents_json, workspaces_json, updated_at";

impl Store {
    /// Materialize a kind:0 profile. Newer `updated_at` wins.
    pub fn upsert_profile(
        &self,
        pubkey: &str,
        name: &str,
        slug: &str,
        host: &str,
        is_backend: bool,
        updated_at: u64,
    ) -> Result<()> {
        self.upsert_profile_with_agent_slug(pubkey, name, slug, "", host, is_backend, updated_at)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_profile_with_agent_slug(
        &self,
        pubkey: &str,
        name: &str,
        slug: &str,
        agent_slug: &str,
        host: &str,
        is_backend: bool,
        updated_at: u64,
    ) -> Result<()> {
        self.upsert_profile_snapshot(
            pubkey,
            name,
            slug,
            agent_slug,
            host,
            is_backend,
            &[],
            &[],
            updated_at,
        )
    }

    /// Materialize one complete kind:0 profile snapshot. Backend agents and
    /// workspaces replace atomically with the profile row.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_profile_snapshot(
        &self,
        pubkey: &str,
        name: &str,
        slug: &str,
        agent_slug: &str,
        host: &str,
        is_backend: bool,
        agents: &[(String, String)],
        workspaces: &[String],
        updated_at: u64,
    ) -> Result<()> {
        let agents_json = serde_json::to_string(agents)?;
        let workspaces_json = serde_json::to_string(workspaces)?;
        self.conn.execute(
            "INSERT INTO relay_profiles
                 (pubkey, name, slug, agent_slug, host, is_backend,
                  agents_json, workspaces_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(pubkey) DO UPDATE SET
                 name=excluded.name, slug=excluded.slug,
                 agent_slug=excluded.agent_slug, host=excluded.host,
                 is_backend=excluded.is_backend,
                 agents_json=excluded.agents_json,
                 workspaces_json=excluded.workspaces_json,
                 updated_at=excluded.updated_at
             WHERE excluded.updated_at >= relay_profiles.updated_at",
            params![
                pubkey,
                name,
                slug,
                agent_slug,
                host,
                is_backend as i64,
                agents_json,
                workspaces_json,
                updated_at
            ],
        )?;
        Ok(())
    }

    /// Fetch one pubkey's profile.
    pub fn get_profile(&self, pubkey: &str) -> Result<Option<Profile>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLS} FROM relay_profiles WHERE pubkey=?1"),
                params![pubkey],
                row_to_profile,
            )
            .optional()?)
    }

    /// Complete management-key backend profiles, ordered by host label.
    pub fn list_backend_profiles(&self) -> Result<Vec<Profile>> {
        let mut statement = self.conn.prepare(&format!(
            "SELECT {COLS} FROM relay_profiles
             WHERE is_backend=1 ORDER BY host, pubkey"
        ))?;
        let rows = statement.query_map([], row_to_profile)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Reverse lookup: the pubkey of an agent advertising `slug` on the exact
    /// config.json `backendName` label. Non-backend profiles only.
    pub fn resolve_agent_pubkey(&self, slug: &str, host: &str) -> Result<Option<String>> {
        let name = crate::idref::agent_label(slug, host);
        let mut stmt = self.conn.prepare(
            "SELECT pubkey, host FROM relay_profiles
                 WHERE is_backend=0 AND agent_slug='' AND (slug=?1 OR name=?2)",
        )?;
        let rows = stmt.query_map(params![slug, name], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (pk, h) = row?;
            if h == host {
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }

    /// Resolve a remote handle only for a pubkey with session-status history.
    /// Expired status remains valid because an offline lease stays addressable
    /// until its owner publishes the replacement profile during reclamation.
    pub fn resolve_profile_handle_pubkey(&self, handle: &str) -> Result<Option<String>> {
        let handle = handle.trim();
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT pubkey FROM relay_profiles
             WHERE is_backend=0 AND agent_slug<>'' AND (name=?1 OR slug=?1)",
        )?;
        let matches = stmt
            .query_map([handle], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let pubkey = match matches.as_slice() {
            [] => return Ok(None),
            [one] => one,
            _ => anyhow::bail!("remote handle {handle:?} is ambiguous"),
        };
        let has_status = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM relay_status WHERE pubkey=?1)",
            [pubkey],
            |row| row.get::<_, bool>(0),
        )?;
        Ok(has_status.then(|| pubkey.clone()))
    }

    /// Reverse lookup: the pubkey of a backend with exactly this config
    /// `backendName` label. Invite/orchestration surfaces do not accept OS/DNS
    /// hostnames, pubkeys, NIP-05, or slugified display strings here.
    pub fn pubkey_for_backend_label(&self, backend_label: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT pubkey, host FROM relay_profiles WHERE is_backend=1")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (pk, h) = row?;
            if h == backend_label {
                return Ok(Some(pk));
            }
        }
        Ok(None)
    }

    /// The agent slug advertised by a pubkey's profile, if any (and non-empty).
    pub fn resolve_slug_for_pubkey(&self, pubkey: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT slug, agent_slug FROM relay_profiles WHERE pubkey=?1",
                params![pubkey],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .map(|(slug, agent_slug)| {
                crate::idref::session_handle_from_profile_name(&slug, &agent_slug)
            })
            .filter(|s| !s.is_empty()))
    }
}
