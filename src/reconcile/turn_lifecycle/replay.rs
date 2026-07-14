use std::collections::BTreeMap;

use anyhow::Result;
use trellis_core::{Graph, GraphResult, Transaction};
use trellis_testing::{DataTransactionScript, TrellisHarness};

use crate::reconcile::journal::InputFact;
use crate::reconcile::labels::NodeLabels;
use crate::reconcile::replay::ReplayReport;

use super::model::{ensure_session, fact_pubkey, stage_fact, SessionNodes};
use super::{TurnCommand, TurnProjectionSeed};

struct ReplayState {
    sessions: BTreeMap<String, SessionNodes>,
    labels: NodeLabels,
}

impl ReplayState {
    fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
            labels: NodeLabels::new(),
        }
    }

    fn apply(
        &mut self,
        operation: &InputFact,
        tx: &mut Transaction<'_, TurnCommand>,
    ) -> GraphResult<()> {
        let Some(pubkey) = fact_pubkey(operation) else {
            return Ok(());
        };
        ensure_session(
            tx,
            &mut self.labels,
            &mut self.sessions,
            &TurnProjectionSeed {
                pubkey: pubkey.to_string(),
                working: false,
                turn_started_at: 0,
                transcript_ref: None,
            },
        )?;
        stage_fact(&self.sessions, operation, tx)
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
    ReplayReport::from_harness("turn_lifecycle", &first, export_trace)
}

fn run(
    script: &DataTransactionScript<InputFact>,
) -> Result<TrellisHarness<Graph<TurnCommand>, TurnCommand>, trellis_testing::ScenarioError> {
    let mut state = ReplayState::new();
    TrellisHarness::replay_data(
        Graph::<TurnCommand>::new_with_command_type,
        script,
        move |operation, tx| state.apply(operation, tx),
    )
}
