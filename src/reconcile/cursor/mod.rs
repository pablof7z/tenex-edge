//! Cursor reconciler: serializes fabric cursor decisions so the host applies
//! a graph-derived `HookFrame` or `NoFrame`, never an independent CAS.

mod model;
pub(crate) mod replay;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use trellis_core::{Graph, GraphResult, ResourceCommand, ResourceCommandCause, TransactionResult};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::{key_path, NodeLabels};

use model::{cursor_key, ensure_session, fact_seed, opts, stage_fact, SessionNodes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorSeed {
    pub session_id: String,
    pub seen_cursor: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CursorFrame {
    HookFrame,
    NoFrame,
}

impl CursorFrame {
    pub fn as_str(&self) -> &'static str {
        match self {
            CursorFrame::HookFrame => "HookFrame",
            CursorFrame::NoFrame => "NoFrame",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorCommand {
    pub session_id: String,
    pub cursor_before: u64,
    pub cursor_after: u64,
    pub delta_since: Option<u64>,
    pub frame: CursorFrame,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CursorEffect {
    Advance {
        session_id: String,
        from: u64,
        to: u64,
        delta_since: u64,
    },
    NoFrame,
}

pub struct CursorOutcome {
    pub effects: Vec<CursorEffect>,
    pub result: TransactionResult<CursorCommand>,
}

pub struct CursorPreview {
    pub labels: NodeLabels,
    pub result: TransactionResult<CursorCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorStateRow {
    pub session: String,
    pub cursor: u64,
    pub last_frame: String,
    pub delta_since: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorWhy {
    pub resource_key: String,
    pub last_kind: String,
    pub cause: String,
    pub input_causes: Vec<String>,
}

#[derive(Clone)]
pub struct CursorReconciler {
    graph: Graph<CursorCommand>,
    sessions: BTreeMap<String, SessionNodes>,
    cursors: BTreeMap<String, u64>,
    last: BTreeMap<String, CursorCommand>,
    labels: NodeLabels,
    next_seq: u64,
}

impl Default for CursorReconciler {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorReconciler {
    pub fn new() -> Self {
        Self {
            graph: Graph::<CursorCommand>::new_with_command_type(),
            sessions: BTreeMap::new(),
            cursors: BTreeMap::new(),
            last: BTreeMap::new(),
            labels: NodeLabels::new(),
            next_seq: 0,
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

    pub fn request(&mut self, seed: CursorSeed, fact: InputFact) -> GraphResult<CursorOutcome> {
        let seq = self.next_seq + 1;
        let (result, _) = self.stage(seed, &fact, seq, false)?;
        self.next_seq = seq;
        let effects = self.translate(&result);
        Ok(CursorOutcome { effects, result })
    }

    pub fn preview_request(
        &mut self,
        seed: CursorSeed,
        fact: &InputFact,
    ) -> GraphResult<CursorPreview> {
        let (result, labels) = self.stage(seed, fact, self.next_seq + 1, true)?;
        Ok(CursorPreview { labels, result })
    }

    pub fn preview_fact(&mut self, fact: &InputFact) -> GraphResult<Option<CursorPreview>> {
        let Some((session_id, observed_cursor)) = fact_seed(fact) else {
            return Ok(None);
        };
        let seed = CursorSeed {
            session_id,
            seen_cursor: observed_cursor,
        };
        self.preview_request(seed, fact).map(Some)
    }

    pub fn state_rows(&self) -> Vec<CursorStateRow> {
        self.last
            .values()
            .map(|cmd| CursorStateRow {
                session: cmd.session_id.clone(),
                cursor: cmd.cursor_after,
                last_frame: cmd.frame.as_str().to_string(),
                delta_since: cmd.delta_since,
            })
            .collect()
    }

    pub fn explain_cursor(&self, id: &str) -> Option<CursorWhy> {
        let why = self.graph.why_resource_command(&cursor_key(id))?;
        let input_causes = self
            .labels
            .labels_for(&why.input_causes)
            .into_iter()
            .filter(|label| !label.ends_with("/request_seq"))
            .collect();
        Some(CursorWhy {
            resource_key: key_path(&cursor_key(id)),
            last_kind: format!("{:?}", why.kind),
            cause: self.cause_label(&why.cause),
            input_causes,
        })
    }

    pub fn assert_oracle(&self) -> GraphResult<()> {
        self.graph.assert_incremental_equals_full()?;
        Ok(())
    }

    fn stage(
        &mut self,
        seed: CursorSeed,
        fact: &InputFact,
        seq: u64,
        preview: bool,
    ) -> GraphResult<(TransactionResult<CursorCommand>, NodeLabels)> {
        let mut sessions = self.sessions.clone();
        let mut labels = self.labels.clone();
        let current_cursor = self.current_cursor(&seed);
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        let nodes = ensure_session(&mut tx, &mut labels, &mut sessions, &seed)?;
        stage_fact(&mut tx, &nodes, current_cursor, fact, seq)?;
        let result = if preview { tx.preview()? } else { tx.commit()? };
        if !preview {
            self.sessions = sessions;
            self.labels = labels.clone();
        }
        Ok((result, labels))
    }

    fn current_cursor(&self, seed: &CursorSeed) -> u64 {
        self.cursors
            .get(&seed.session_id)
            .copied()
            .unwrap_or(seed.seen_cursor)
    }

    fn translate(&mut self, result: &TransactionResult<CursorCommand>) -> Vec<CursorEffect> {
        let mut effects = Vec::new();
        for command in result.resource_plan.commands() {
            let cmd = match command {
                ResourceCommand::Open { command, .. }
                | ResourceCommand::Replace { command, .. }
                | ResourceCommand::Refresh { command, .. } => command,
                ResourceCommand::Close { .. } => continue,
            };
            self.cursors
                .insert(cmd.session_id.clone(), cmd.cursor_after);
            self.last.insert(cmd.session_id.clone(), cmd.clone());
            match cmd.frame {
                CursorFrame::HookFrame => effects.push(CursorEffect::Advance {
                    session_id: cmd.session_id.clone(),
                    from: cmd.cursor_before,
                    to: cmd.cursor_after,
                    delta_since: cmd.delta_since.unwrap_or(cmd.cursor_before),
                }),
                CursorFrame::NoFrame => effects.push(CursorEffect::NoFrame),
            }
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
