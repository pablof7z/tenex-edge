//! Durable replay guards for backend-owned side effects.
//!
//! These claims are deliberately separate from agent inbox delivery: a claim
//! key identifies an operation target, never a session or agent identity.

use super::*;

const PROCESSING_LEASE_SECS: u64 = 10 * 60;

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
        self.complete_orchestration_target(event_id, &offline_mention_claim(mentioned_pubkey), now)
    }
}

fn offline_mention_claim(mentioned_pubkey: &str) -> String {
    format!("offline-mention:{mentioned_pubkey}")
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
