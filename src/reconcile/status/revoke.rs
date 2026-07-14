use super::*;

impl StatusReconciler {
    /// Explicit operator destruction closes the status resource immediately.
    pub fn on_session_revoked(&mut self, id: &str, now: u64) -> GraphResult<StatusOutcome> {
        let Some(nodes) = self.sessions.get(id).copied() else {
            return self.empty_commit();
        };
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        tx.close_scope(nodes.scope)?;
        let result = tx.commit()?;
        drop(tx);
        self.sessions.remove(id);
        let effects = self.translate(&result, now);
        Ok(StatusOutcome { effects, result })
    }
}
