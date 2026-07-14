use trellis_core::{GraphResult, ResourceCommandCause, ScopeId, TransactionResult};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::{key_path, NodeLabels};

use super::{watch_key, ReconcileCommand, Reconciler};

pub struct SessionWatchPreview {
    pub labels: NodeLabels,
    pub result: TransactionResult<ReconcileCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionWatchStateRow {
    pub session: String,
    pub resource_key: String,
    pub refcount: usize,
    pub owners: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionWatchWhy {
    pub resource_key: String,
    pub last_kind: String,
    pub cause: String,
    pub input_causes: Vec<String>,
}

impl Reconciler {
    pub fn preview_fact(&mut self, fact: &InputFact) -> GraphResult<Option<SessionWatchPreview>> {
        if !session_watch_fact(fact) {
            return Ok(None);
        }
        let live = self.next_live_sessions(fact);
        let mut tx = self.graph.begin_transaction()?;
        tx.set_input(self.live_sessions, live)?;
        let result = tx.preview()?;
        Ok(Some(SessionWatchPreview {
            labels: self.labels.clone(),
            result,
        }))
    }

    pub fn revision(&self) -> u64 {
        self.graph.revision().get()
    }

    pub fn graph_node_count(&self) -> usize {
        self.graph.nodes().count()
    }

    pub fn state_rows(&self) -> Vec<SessionWatchStateRow> {
        let mut sessions = self.current_live_sessions().into_iter().collect::<Vec<_>>();
        sessions.sort();
        sessions
            .into_iter()
            .map(|session| {
                let key = watch_key(&session);
                let owners: Vec<String> = self
                    .graph
                    .resource_owners(&key)
                    .map(|scopes| {
                        scopes
                            .iter()
                            .map(|scope| self.scope_label(*scope))
                            .collect()
                    })
                    .unwrap_or_default();
                SessionWatchStateRow {
                    session,
                    resource_key: key_path(&key),
                    refcount: owners.len(),
                    owners,
                }
            })
            .collect()
    }

    pub fn explain_watch(&self, pubkey: &str) -> Option<SessionWatchWhy> {
        let key = watch_key(pubkey);
        let why = self.graph.why_resource_command(&key)?;
        let cause = self.cause_label(&why.cause);
        let mut input_causes = self.labels.labels_for(&why.input_causes);
        if input_causes.is_empty() || input_causes.iter().all(|label| label.starts_with("node:")) {
            input_causes = vec!["session_watch/resources".to_string()];
        }
        Some(SessionWatchWhy {
            resource_key: key_path(&key),
            last_kind: format!("{:?}", why.kind),
            cause,
            input_causes,
        })
    }

    fn cause_label(&self, cause: &ResourceCommandCause) -> String {
        match cause {
            ResourceCommandCause::Planner { collection } => format!(
                "planner: {}",
                self.labels
                    .label_of(*collection)
                    .map(str::to_string)
                    .unwrap_or_else(|| "session_watch/resources".to_string())
            ),
            ResourceCommandCause::ScopeClosed { scope } => {
                format!("scope-closed: {}", self.scope_label(*scope))
            }
        }
    }

    fn scope_label(&self, scope: ScopeId) -> String {
        self.graph
            .scope_meta(scope)
            .map(|m| m.debug_name().to_string())
            .unwrap_or_else(|| format!("scope:{}", scope.get()))
    }
}

fn session_watch_fact(fact: &InputFact) -> bool {
    matches!(
        fact,
        InputFact::SessionStarted { .. }
            | InputFact::ProcessExited {
                pubkey: Some(_),
                ..
            }
    )
}
