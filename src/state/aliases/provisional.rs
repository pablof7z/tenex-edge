//! Attempt-owned alias writes and exact rollback.

use super::*;
use std::collections::{HashMap, HashSet};

type AliasKey = (String, String, String);

#[derive(Default)]
pub(crate) struct AliasWriteState {
    stacks: HashMap<AliasKey, Vec<AliasWriteFrame>>,
    attempts: HashMap<String, AttemptState>,
}

struct AliasWriteFrame {
    owner: String,
    prior: Option<SessionAlias>,
    written_session_id: String,
    invalidated: bool,
}

#[derive(Clone, Copy)]
enum AttemptState {
    Active,
    Aborted,
    Committed,
}

impl Store {
    /// Capture the current row and replace it as one store-locked operation.
    /// The in-memory frame stack is only rollback coordination for active
    /// session-start requests; `session_aliases` remains the durable locator map.
    pub(crate) fn put_alias_provisional(
        &self,
        harness: &str,
        external_id_kind: &str,
        external_id: &str,
        session_id: &str,
        created_at: u64,
        owner: &str,
    ) -> Result<()> {
        let key = alias_key(harness, external_id_kind, external_id);
        let same_owner = self
            .alias_writes
            .borrow()
            .stacks
            .get(&key)
            .and_then(|stack| stack.last())
            .is_some_and(|frame| frame.owner == owner);
        let prior = (!same_owner)
            .then(|| self.alias_for_key(harness, external_id_kind, external_id))
            .transpose()?
            .flatten();
        self.write_alias(
            harness,
            external_id_kind,
            external_id,
            session_id,
            created_at,
        )?;
        let mut writes = self.alias_writes.borrow_mut();
        writes
            .attempts
            .entry(owner.to_string())
            .or_insert(AttemptState::Active);
        if same_owner {
            if let Some(frame) = writes
                .stacks
                .get_mut(&key)
                .and_then(|stack| stack.last_mut())
            {
                frame.written_session_id = session_id.to_string();
            }
        } else {
            writes.stacks.entry(key).or_default().push(AliasWriteFrame {
                owner: owner.to_string(),
                prior,
                written_session_id: session_id.to_string(),
                invalidated: false,
            });
        }
        Ok(())
    }

    pub(crate) fn abort_alias_attempt(&self, owner: &str) -> Result<()> {
        self.settle_alias_attempt(owner, AttemptState::Aborted)
    }

    pub(crate) fn commit_alias_attempt(&self, owner: &str) -> Result<()> {
        self.settle_alias_attempt(owner, AttemptState::Committed)
    }

    pub(super) fn clear_alias_write_stack(
        &self,
        harness: &str,
        external_id_kind: &str,
        external_id: &str,
    ) {
        self.alias_writes.borrow_mut().stacks.remove(&alias_key(
            harness,
            external_id_kind,
            external_id,
        ));
        self.prune_attempt_states();
    }

    pub(super) fn invalidate_alias_target(
        &self,
        session_id: &str,
        external_id_kind: &str,
    ) -> Result<()> {
        let keys = {
            let mut writes = self.alias_writes.borrow_mut();
            for (key, stack) in &mut writes.stacks {
                if key.1 != external_id_kind {
                    continue;
                }
                for frame in stack {
                    if frame
                        .prior
                        .as_ref()
                        .is_some_and(|alias| alias.session_id == session_id)
                    {
                        frame.prior = None;
                    }
                    if frame.written_session_id == session_id {
                        frame.invalidated = true;
                    }
                }
            }
            writes.stacks.keys().cloned().collect::<Vec<_>>()
        };
        for key in keys {
            self.settle_alias_key(&key)?;
        }
        self.prune_attempt_states();
        Ok(())
    }

    fn settle_alias_attempt(&self, owner: &str, state: AttemptState) -> Result<()> {
        self.alias_writes
            .borrow_mut()
            .attempts
            .insert(owner.to_string(), state);
        let keys = self
            .alias_writes
            .borrow()
            .stacks
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            self.settle_alias_key(&key)?;
        }
        self.prune_attempt_states();
        Ok(())
    }

    fn settle_alias_key(&self, key: &AliasKey) -> Result<()> {
        loop {
            let action = {
                let writes = self.alias_writes.borrow();
                let Some(frame) = writes.stacks.get(key).and_then(|stack| stack.last()) else {
                    return Ok(());
                };
                (
                    if frame.invalidated {
                        AttemptState::Aborted
                    } else {
                        writes
                            .attempts
                            .get(&frame.owner)
                            .copied()
                            .unwrap_or(AttemptState::Active)
                    },
                    frame.prior.clone(),
                )
            };
            match action {
                (AttemptState::Active, _) => return Ok(()),
                (AttemptState::Committed, _) => {
                    self.alias_writes.borrow_mut().stacks.remove(key);
                    return Ok(());
                }
                (AttemptState::Aborted, prior) => {
                    self.restore_alias_row(key, prior.as_ref())?;
                    let mut writes = self.alias_writes.borrow_mut();
                    if let Some(stack) = writes.stacks.get_mut(key) {
                        stack.pop();
                        if stack.is_empty() {
                            writes.stacks.remove(key);
                        }
                    }
                }
            }
        }
    }

    fn restore_alias_row(&self, key: &AliasKey, prior: Option<&SessionAlias>) -> Result<()> {
        match prior {
            Some(alias) => self.write_alias(
                &alias.harness,
                &alias.external_id_kind,
                &alias.external_id,
                &alias.session_id,
                alias.created_at,
            ),
            None => {
                self.conn.execute(
                    "DELETE FROM session_aliases
                     WHERE harness=?1 AND external_id_kind=?2 AND external_id=?3",
                    params![key.0, key.1, key.2],
                )?;
                Ok(())
            }
        }
    }

    fn prune_attempt_states(&self) {
        let mut writes = self.alias_writes.borrow_mut();
        let referenced = writes
            .stacks
            .values()
            .flatten()
            .map(|frame| frame.owner.clone())
            .collect::<HashSet<_>>();
        writes
            .attempts
            .retain(|owner, _| referenced.contains(owner));
    }
}

fn alias_key(harness: &str, kind: &str, external_id: &str) -> AliasKey {
    (
        harness.to_string(),
        kind.to_string(),
        external_id.to_string(),
    )
}
