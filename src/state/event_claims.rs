//! Durable replay guards for backend-owned side effects.
//!
//! These claims are deliberately separate from agent inbox delivery: a claim
//! key identifies an operation target, never a session or agent identity.

use super::*;

const PROCESSING_LEASE_SECS: u64 = 10 * 60;
const OFFLINE_MENTION_RETRY_DELAY_SECS: u64 = 30;
pub(super) const OFFLINE_MENTION_CLAIM_PREFIX: &str = "offline-mention:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OfflineMentionClaim {
    pub(crate) event_id: String,
    pub(crate) mentioned_pubkey: String,
    pub(crate) from_pubkey: String,
    pub(crate) channel_h: String,
    pub(crate) body: String,
}

impl Store {
    /// Claim one operation target for processing. Completed targets and live
    /// in-flight claims return `false`; failed targets return to `pending` and
    /// can be claimed again.
    pub fn claim_orchestration_target(
        &self,
        event_id: &str,
        claim_key: &str,
        from_pubkey: &str,
        channel_h: &str,
        body: &str,
        now: u64,
    ) -> Result<bool> {
        let stale_before = now.saturating_sub(PROCESSING_LEASE_SECS);
        let n = self.conn.execute(
            "INSERT INTO event_claims
                 (event_id, claim_key, state, from_pubkey, channel_h, body, created_at, updated_at)
             VALUES (?1, ?2, 'processing', ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(event_id, claim_key) DO UPDATE SET
                 state='processing',
                 from_pubkey=excluded.from_pubkey,
                 channel_h=excluded.channel_h,
                 body=excluded.body,
                 updated_at=excluded.updated_at
             WHERE event_claims.state='pending'
                OR (event_claims.state='processing' AND event_claims.updated_at < ?7)",
            params![
                event_id,
                claim_key,
                from_pubkey,
                channel_h,
                body,
                now,
                stale_before
            ],
        )?;
        Ok(n > 0)
    }

    pub fn complete_orchestration_target(
        &self,
        event_id: &str,
        claim_key: &str,
        now: u64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE event_claims SET state='completed', updated_at=?3
             WHERE event_id=?1 AND claim_key=?2",
            params![event_id, claim_key, now],
        )?;
        Ok(())
    }

    pub fn retry_orchestration_target(&self, event_id: &str, claim_key: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE event_claims SET state='pending', updated_at=0
             WHERE event_id=?1 AND claim_key=?2 AND state='processing'",
            params![event_id, claim_key],
        )?;
        Ok(())
    }

    pub fn claim_management_command(
        &self,
        event_id: &str,
        from_pubkey: &str,
        channel_h: &str,
        body: &str,
        now: u64,
    ) -> Result<bool> {
        self.claim_orchestration_target(event_id, "management", from_pubkey, channel_h, body, now)
    }

    pub fn complete_management_command(&self, event_id: &str, now: u64) -> Result<()> {
        self.complete_orchestration_target(event_id, "management", now)
    }

    pub fn claim_offline_mention(
        &self,
        event_id: &str,
        mentioned_pubkey: &str,
        from_pubkey: &str,
        channel_h: &str,
        body: &str,
        now: u64,
    ) -> Result<bool> {
        self.claim_orchestration_target(
            event_id,
            &offline_mention_claim(mentioned_pubkey),
            from_pubkey,
            channel_h,
            body,
            now,
        )
    }

    pub fn complete_offline_mention(
        &self,
        event_id: &str,
        mentioned_pubkey: &str,
        now: u64,
    ) -> Result<()> {
        // Nostr observations may replay an event long after the local message
        // cache is pruned. Keep a compact tombstone for this process-spawning
        // side effect instead of applying the generic completed-ledger TTL.
        self.conn.execute(
            "UPDATE event_claims
             SET state='completed', from_pubkey='', channel_h='', body='', updated_at=?3
             WHERE event_id=?1 AND claim_key=?2",
            params![event_id, offline_mention_claim(mentioned_pubkey), now],
        )?;
        Ok(())
    }

    /// Return a claimed offline mention to the retryable state. Recovery owns
    /// the side effect; inbox delivery remains independently pending under the
    /// exact addressed pubkey.
    pub fn retry_offline_mention(
        &self,
        event_id: &str,
        mentioned_pubkey: &str,
        now: u64,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE event_claims SET state='pending', updated_at=?3
             WHERE event_id=?1 AND claim_key=?2 AND state='processing'",
            params![event_id, offline_mention_claim(mentioned_pubkey), now],
        )?;
        Ok(())
    }

    /// Bounded durable recovery work. Fresh failures wait briefly before retry;
    /// abandoned processing leases become eligible after the crash lease.
    pub(crate) fn list_retryable_offline_mentions(
        &self,
        now: u64,
        limit: u32,
    ) -> Result<Vec<OfflineMentionClaim>> {
        let retry_before = now.saturating_sub(OFFLINE_MENTION_RETRY_DELAY_SECS);
        let stale_before = now.saturating_sub(PROCESSING_LEASE_SECS);
        let prefix = OFFLINE_MENTION_CLAIM_PREFIX;
        let mut statement = self.conn.prepare(
            "SELECT event_id, substr(claim_key, length(?1) + 1),
                    from_pubkey, channel_h, body
             FROM event_claims
             WHERE claim_key LIKE ?2
               AND ((state='pending' AND updated_at<=?3)
                 OR (state='processing' AND updated_at<?4))
             ORDER BY updated_at, created_at, event_id
             LIMIT ?5",
        )?;
        let rows = statement.query_map(
            params![
                prefix,
                format!("{prefix}%"),
                retry_before,
                stale_before,
                limit
            ],
            |row| {
                Ok(OfflineMentionClaim {
                    event_id: row.get(0)?,
                    mentioned_pubkey: row.get(1)?,
                    from_pubkey: row.get(2)?,
                    channel_h: row.get(3)?,
                    body: row.get(4)?,
                })
            },
        )?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}

fn offline_mention_claim(mentioned_pubkey: &str) -> String {
    format!("{OFFLINE_MENTION_CLAIM_PREFIX}{mentioned_pubkey}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_for(store: &Store, event_id: &str, claim_key: &str) -> String {
        store
            .conn
            .query_row(
                "SELECT state FROM event_claims WHERE event_id=?1 AND claim_key=?2",
                params![event_id, claim_key],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn orchestration_claim_retries_only_failed_targets() {
        let store = Store::open_memory().unwrap();
        let a = "orchestration:backend:0:a";
        let b = "orchestration:backend:1:b";
        assert!(store
            .claim_orchestration_target("ev", a, "admin", "child", "a", 10)
            .unwrap());
        assert!(store
            .claim_orchestration_target("ev", b, "admin", "child", "b", 10)
            .unwrap());

        store.retry_orchestration_target("ev", a).unwrap();
        store.complete_orchestration_target("ev", b, 11).unwrap();

        assert!(store
            .claim_orchestration_target("ev", a, "admin", "child", "a", 12)
            .unwrap());
        assert!(!store
            .claim_orchestration_target("ev", b, "admin", "child", "b", 12)
            .unwrap());
        assert_eq!(state_for(&store, "ev", a), "processing");
        assert_eq!(state_for(&store, "ev", b), "completed");
    }
}
