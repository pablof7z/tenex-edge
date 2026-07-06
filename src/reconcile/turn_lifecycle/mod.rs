//! Turn lifecycle projection: canonical turn facts derive the local session-row
//! fields the host must apply (`working`, `turn_started_at`, transcript pointer).

mod model;
pub(crate) mod replay;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use trellis_core::{Graph, GraphResult, ResourceCommand, ResourceCommandCause, TransactionResult};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::{key_path, NodeLabels};

use model::{ensure_session, fact_session_id, opts, stage_fact, turn_key, SessionNodes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnProjectionSeed {
    pub session_id: String,
    pub working: bool,
    pub turn_started_at: u64,
    pub transcript_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnCommand {
    pub session_id: String,
    pub working: bool,
    pub turn_started_at: u64,
    pub transcript_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEffect {
    Apply(TurnCommand),
}

pub struct TurnLifecycleOutcome {
    pub effects: Vec<TurnEffect>,
    pub result: TransactionResult<TurnCommand>,
}

pub struct TurnLifecyclePreview {
    pub labels: NodeLabels,
    pub result: TransactionResult<TurnCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnStateRow {
    pub session: String,
    pub working: bool,
    pub turn_started_at: u64,
    pub transcript_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnWhy {
    pub resource_key: String,
    pub last_kind: String,
    pub cause: String,
    pub input_causes: Vec<String>,
}

#[derive(Clone)]
pub struct TurnLifecycleReconciler {
    graph: Graph<TurnCommand>,
    sessions: BTreeMap<String, SessionNodes>,
    last: BTreeMap<String, TurnCommand>,
    labels: NodeLabels,
}

impl Default for TurnLifecycleReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl TurnLifecycleReconciler {
    pub fn new() -> Self {
        Self {
            graph: Graph::<TurnCommand>::new_with_command_type(),
            sessions: BTreeMap::new(),
            last: BTreeMap::new(),
            labels: NodeLabels::new(),
        }
    }

    pub fn labels(&self) -> &NodeLabels {
        &self.labels
    }

    pub fn revision(&self) -> u64 {
        self.graph.revision().get()
    }

    pub fn graph_node_count(&self) -> usize {
        self.graph.nodes().count()
    }

    pub fn on_turn_started(
        &mut self,
        seed: TurnProjectionSeed,
        at: u64,
        transcript_ref: Option<String>,
    ) -> GraphResult<TurnLifecycleOutcome> {
        let mut facts = vec![InputFact::TurnStarted {
            session_id: seed.session_id.clone(),
            at,
        }];
        if let Some(window_hash) = transcript_ref {
            facts.push(InputFact::TranscriptWindowCaptured {
                session_id: seed.session_id.clone(),
                window_hash,
                at,
            });
        }
        self.commit_facts(seed, facts)
    }

    pub fn on_turn_ended(
        &mut self,
        seed: TurnProjectionSeed,
        at: u64,
    ) -> GraphResult<TurnLifecycleOutcome> {
        self.commit_facts(
            seed.clone(),
            vec![InputFact::TurnEnded {
                session_id: seed.session_id,
                at,
            }],
        )
    }

    pub fn preview_turn_started(
        &mut self,
        seed: TurnProjectionSeed,
        at: u64,
        transcript_ref: Option<String>,
    ) -> GraphResult<TurnLifecyclePreview> {
        let mut facts = vec![InputFact::TurnStarted {
            session_id: seed.session_id.clone(),
            at,
        }];
        if let Some(window_hash) = transcript_ref {
            facts.push(InputFact::TranscriptWindowCaptured {
                session_id: seed.session_id.clone(),
                window_hash,
                at,
            });
        }
        self.preview_facts(seed, &facts)
    }

    pub fn preview_turn_ended(
        &mut self,
        seed: TurnProjectionSeed,
        at: u64,
    ) -> GraphResult<TurnLifecyclePreview> {
        self.preview_facts(
            seed.clone(),
            &[InputFact::TurnEnded {
                session_id: seed.session_id,
                at,
            }],
        )
    }

    pub fn preview_fact(&mut self, fact: &InputFact) -> GraphResult<Option<TurnLifecyclePreview>> {
        let Some(session_id) = fact_session_id(fact) else {
            return Ok(None);
        };
        let seed = self.seed_from_live(session_id);
        self.preview_facts(seed, std::slice::from_ref(fact))
            .map(Some)
    }

    pub fn state_rows(&self) -> Vec<TurnStateRow> {
        self.last
            .values()
            .map(|cmd| TurnStateRow {
                session: cmd.session_id.clone(),
                working: cmd.working,
                turn_started_at: cmd.turn_started_at,
                transcript_ref: cmd.transcript_ref.clone(),
            })
            .collect()
    }

    pub fn explain_turn(&self, id: &str) -> Option<TurnWhy> {
        let why = self.graph.why_resource_command(&turn_key(id))?;
        Some(TurnWhy {
            resource_key: key_path(&turn_key(id)),
            last_kind: format!("{:?}", why.kind),
            cause: self.cause_label(&why.cause),
            input_causes: self.labels.labels_for(&why.input_causes),
        })
    }

    pub fn assert_oracle(&self) -> GraphResult<()> {
        self.graph.assert_incremental_equals_full()?;
        Ok(())
    }

    fn seed_from_live(&self, session_id: &str) -> TurnProjectionSeed {
        self.last
            .get(session_id)
            .map(|cmd| TurnProjectionSeed {
                session_id: cmd.session_id.clone(),
                working: cmd.working,
                turn_started_at: cmd.turn_started_at,
                transcript_ref: cmd.transcript_ref.clone(),
            })
            .unwrap_or_else(|| TurnProjectionSeed {
                session_id: session_id.to_string(),
                working: false,
                turn_started_at: 0,
                transcript_ref: None,
            })
    }

    fn commit_facts(
        &mut self,
        seed: TurnProjectionSeed,
        facts: Vec<InputFact>,
    ) -> GraphResult<TurnLifecycleOutcome> {
        let mut sessions = self.sessions.clone();
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        ensure_session(&mut tx, &mut self.labels, &mut sessions, &seed)?;
        for fact in &facts {
            stage_fact(&sessions, fact, &mut tx)?;
        }
        let result = tx.commit()?;
        drop(tx);
        self.sessions = sessions;
        let effects = self.translate(&result);
        Ok(TurnLifecycleOutcome { effects, result })
    }

    fn preview_facts(
        &mut self,
        seed: TurnProjectionSeed,
        facts: &[InputFact],
    ) -> GraphResult<TurnLifecyclePreview> {
        let mut sessions = self.sessions.clone();
        let mut labels = self.labels.clone();
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        ensure_session(&mut tx, &mut labels, &mut sessions, &seed)?;
        for fact in facts {
            stage_fact(&sessions, fact, &mut tx)?;
        }
        let result = tx.preview()?;
        Ok(TurnLifecyclePreview { labels, result })
    }

    fn translate(&mut self, result: &TransactionResult<TurnCommand>) -> Vec<TurnEffect> {
        let mut effects = Vec::new();
        for command in result.resource_plan.commands() {
            let cmd = match command {
                ResourceCommand::Open { command, .. }
                | ResourceCommand::Replace { command, .. }
                | ResourceCommand::Refresh { command, .. } => command,
                ResourceCommand::Close { key, .. } => {
                    if let Some(id) = key.segment(1) {
                        self.last.remove(id);
                    }
                    continue;
                }
            };
            self.last.insert(cmd.session_id.clone(), cmd.clone());
            effects.push(TurnEffect::Apply(cmd.clone()));
        }
        effects
    }

    fn cause_label(&self, cause: &ResourceCommandCause) -> String {
        match cause {
            ResourceCommandCause::Planner { collection } => format!(
                "planner: {}",
                self.labels
                    .label_of(*collection)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("node:{}", collection.get()))
            ),
            ResourceCommandCause::ScopeClosed { scope } => {
                format!("scope-closed: {}", scope.get())
            }
        }
    }
}
