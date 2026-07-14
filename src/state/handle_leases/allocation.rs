//! Atomic public-handle and ordinary-session identity allocation.

use super::*;
use rusqlite::{Transaction, TransactionBehavior};

impl Store {
    #[cfg(test)]
    pub(crate) fn allocate_handle(
        &self,
        pubkey: &str,
        agent_slug: &str,
        now: u64,
    ) -> Result<HandleAllocation> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let allocation = allocate_handle_in(&tx, pubkey, agent_slug, now)?;
        tx.commit()?;
        Ok(allocation)
    }

    #[cfg(test)]
    pub(crate) fn allocate_custom_handle(
        &self,
        pubkey: &str,
        agent_slug: &str,
        name: &str,
        now: u64,
    ) -> Result<HandleAllocation> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let allocation = allocate_custom_handle_in(&tx, pubkey, agent_slug, name, now)?;
        tx.commit()?;
        Ok(allocation)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_ordinary_identity<T>(
        &self,
        session_id: &str,
        agent_slug: &str,
        channel: &str,
        native_id: &str,
        session_name: Option<&str>,
        now: u64,
        derive: impl FnOnce(&str) -> Result<(T, String)>,
    ) -> Result<(T, String, HandleAllocation)> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let prior_pubkey = tx
            .query_row(
                "SELECT pubkey FROM identities WHERE session_id=?1 LIMIT 1",
                [session_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let signer_salt = match prior_pubkey.as_deref() {
            Some(pubkey) => tx
                .query_row(
                    "SELECT signer_salt FROM session_signers WHERE pubkey=?1",
                    [pubkey],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .with_context(|| format!("pubkey {pubkey:?} has no signer material"))?,
            None => crate::identity::new_session_signer_salt(),
        };
        let (value, pubkey) = derive(&signer_salt)?;
        if prior_pubkey.as_deref().is_some_and(|prior| prior != pubkey) {
            anyhow::bail!("stored signer material does not reproduce session pubkey");
        }
        tx.execute(
            "INSERT OR IGNORE INTO session_signers (pubkey, signer_salt) VALUES (?1, ?2)",
            params![pubkey, signer_salt],
        )?;
        let stored_salt: String = tx.query_row(
            "SELECT signer_salt FROM session_signers WHERE pubkey=?1",
            [&pubkey],
            |row| row.get(0),
        )?;
        if stored_salt != signer_salt {
            anyhow::bail!("signer material changed for pubkey {pubkey:?}");
        }
        let allocation = match session_name {
            Some(name) => allocate_custom_handle_in(&tx, &pubkey, agent_slug, name, now)?,
            None => allocate_handle_in(&tx, &pubkey, agent_slug, now)?,
        };
        tx.execute(
            "DELETE FROM identities WHERE session_id=?1 AND pubkey<>?2",
            params![session_id, pubkey],
        )?;
        tx.execute(
            "INSERT INTO identities
                 (pubkey, agent_slug, codename, session_id, channel_h, native_id,
                  alive, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)
             ON CONFLICT(pubkey, session_id) DO UPDATE SET
                 agent_slug=excluded.agent_slug, codename=excluded.codename,
                 channel_h=excluded.channel_h, native_id=excluded.native_id, alive=1",
            params![
                pubkey,
                agent_slug,
                allocation.codename,
                session_id,
                channel,
                native_id,
                now
            ],
        )?;
        tx.commit()?;
        Ok((value, pubkey, allocation))
    }
}

fn allocate_handle_in(
    tx: &Transaction<'_>,
    pubkey: &str,
    agent_slug: &str,
    now: u64,
) -> Result<HandleAllocation> {
    if let Some((handle, codename)) = lease_for_pubkey(tx, pubkey)? {
        tx.execute(
            "UPDATE handle_leases SET live=1, last_active_at=?2 WHERE pubkey=?1",
            params![pubkey, now],
        )?;
        return Ok(HandleAllocation {
            handle,
            codename,
            reclaimed_pubkey: None,
        });
    }
    for codename in candidates(pubkey) {
        let handle = crate::idref::session_handle(agent_slug, &codename);
        if remote_profiles::reserves_handle(tx, &handle, Some(pubkey))? {
            continue;
        }
        let occupied = tx
            .query_row(
                "SELECT pubkey, live, last_active_at FROM handle_leases WHERE handle=?1",
                [&handle],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let reclaimed_pubkey = match occupied {
            None => None,
            Some((_owner, true, _)) => continue,
            Some((_owner, false, last_active))
                if now.saturating_sub(last_active) < HANDLE_LEASE_GRACE_SECS =>
            {
                continue;
            }
            Some((owner, false, _)) => Some(owner),
        };
        if let Some(old_pubkey) = reclaimed_pubkey.as_deref() {
            tx.execute(
                "UPDATE identities SET codename='' WHERE pubkey=?1",
                [old_pubkey],
            )?;
            tx.execute("DELETE FROM handle_leases WHERE handle=?1", [&handle])?;
        }
        tx.execute(
            "INSERT INTO handle_leases
                (handle, pubkey, agent_slug, leased_at, last_active_at, live)
             VALUES (?1, ?2, ?3, ?4, ?4, 1)",
            params![handle, pubkey, agent_slug, now],
        )?;
        return Ok(HandleAllocation {
            handle,
            codename,
            reclaimed_pubkey,
        });
    }
    anyhow::bail!("session handle space exhausted for agent {agent_slug:?}")
}

fn allocate_custom_handle_in(
    tx: &Transaction<'_>,
    pubkey: &str,
    agent_slug: &str,
    name: &str,
    now: u64,
) -> Result<HandleAllocation> {
    let (handle, codename) = custom_handle(agent_slug, name)?;
    if let Some((existing, codename)) = lease_for_pubkey(tx, pubkey)? {
        if existing != handle {
            anyhow::bail!(
                "session {pubkey:?} already uses {existing:?}, not requested name {name:?}"
            );
        }
        tx.execute(
            "UPDATE handle_leases SET live=1, last_active_at=?2 WHERE pubkey=?1",
            params![pubkey, now],
        )?;
        return Ok(HandleAllocation {
            handle: existing,
            codename,
            reclaimed_pubkey: None,
        });
    }
    if remote_profiles::reserves_handle(tx, &handle, Some(pubkey))?
        || tx
            .query_row(
                "SELECT 1 FROM handle_leases WHERE handle=?1",
                [&handle],
                |_| Ok(()),
            )
            .optional()?
            .is_some()
    {
        anyhow::bail!("session name {name:?} is already in use as {handle:?}");
    }
    tx.execute(
        "INSERT INTO handle_leases
            (handle, pubkey, agent_slug, leased_at, last_active_at, live)
         VALUES (?1, ?2, ?3, ?4, ?4, 1)",
        params![handle, pubkey, agent_slug, now],
    )?;
    Ok(HandleAllocation {
        handle,
        codename,
        reclaimed_pubkey: None,
    })
}

fn lease_for_pubkey(tx: &Transaction<'_>, pubkey: &str) -> Result<Option<(String, String)>> {
    let row = tx
        .query_row(
            "SELECT handle, agent_slug FROM handle_leases WHERE pubkey=?1",
            [pubkey],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    Ok(row.map(|(handle, agent_slug)| {
        let suffix = format!("-{agent_slug}");
        let codename = handle.strip_suffix(&suffix).unwrap_or(&handle).to_string();
        (handle, codename)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_conflict_rolls_back_signer_and_identity() {
        let store = Store::open_memory().unwrap();
        store
            .allocate_custom_handle("owner", "codex", "named", 1)
            .unwrap();
        let error = store
            .reserve_ordinary_identity("run", "codex", "root", "native", Some("named"), 2, |_| {
                Ok(((), "new-pk".to_string()))
            })
            .unwrap_err();

        assert!(error.to_string().contains("already in use"));
        assert!(store.session_signer_salt("new-pk").unwrap().is_none());
        assert!(store.get_identity("new-pk").unwrap().is_none());
        assert_eq!(
            store.pubkey_for_handle("named-codex").unwrap().as_deref(),
            Some("owner")
        );
    }
}
