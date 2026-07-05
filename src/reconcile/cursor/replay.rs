use std::collections::BTreeMap;

use anyhow::Result;
use trellis_core::{Graph, GraphResult, Transaction};
use trellis_testing::{DataTransactionScript, TrellisHarness};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::NodeLabels;
use crate::reconcile::replay::ReplayReport;

use super::model::{ensure_session, fact_seed, stage_fact, SessionNodes};
use super::{CursorCommand, CursorSeed};

struct ReplayState {
    sessions: BTreeMap<String, SessionNodes>,
    cursors: BTreeMap<String, u64>,
    labels: NodeLabels,
    seq: u64,
}

impl ReplayState {
    fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
            cursors: BTreeMap::new(),
            labels: NodeLabels::new(),
            seq: 0,
        }
    }

    fn apply(
        &mut self,
        operation: &InputFact,
        tx: &mut Transaction<'_, CursorCommand>,
    ) -> GraphResult<()> {
        let Some((session_id, observed_cursor)) = fact_seed(operation) else {
            return Ok(());
        };
        let current = self
            .cursors
            .get(&session_id)
            .copied()
            .unwrap_or(observed_cursor);
        let nodes = ensure_session(
            tx,
            &mut self.labels,
            &mut self.sessions,
            &CursorSeed {
                session_id: session_id.clone(),
                seen_cursor: current,
            },
        )?;
        self.seq += 1;
        stage_fact(tx, &nodes, current, operation, self.seq)?;
        if let InputFact::TurnCheckRequested {
            observed_cursor,
            working,
            at,
            ..
        } = operation
        {
            if *working && *observed_cursor == current && *at > current {
                self.cursors.insert(session_id, *at);
            }
        }
        Ok(())
    }
}

pub(crate) fn replay_script(
    script: &DataTransactionScript<InputFact>,
    export_trace: bool,
) -> Result<ReplayReport> {
    let first = run(script).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let second = run(script).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    first
        .assert_replay_matches(&second)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    ReplayReport::from_harness("cursor", &first, export_trace)
}

fn run(
    script: &DataTransactionScript<InputFact>,
) -> Result<TrellisHarness<Graph<CursorCommand>, CursorCommand>, trellis_testing::ScenarioError> {
    let mut state = ReplayState::new();
    TrellisHarness::replay_data(
        Graph::<CursorCommand>::new_with_command_type,
        script,
        move |operation, tx| state.apply(operation, tx),
    )
}
