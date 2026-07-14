use std::collections::BTreeSet;

use anyhow::Result;
use trellis_core::{DependencyList, Graph, GraphResult, InputNode, ResourcePlan, Transaction};
use trellis_testing::{DataTransactionScript, TrellisHarness};

use crate::reconcile::journal::InputFact;
use crate::reconcile::replay::ReplayReport;

use super::{watch_key, ReconcileCommand};

#[derive(Clone, Copy)]
struct Handles {
    live_sessions: InputNode<BTreeSet<String>>,
}

#[derive(Default)]
struct ReplayState {
    handles: Option<Handles>,
    live_sessions: BTreeSet<String>,
}

impl ReplayState {
    fn apply(
        &mut self,
        operation: &InputFact,
        tx: &mut Transaction<'_, ReconcileCommand>,
    ) -> GraphResult<()> {
        let handles = match self.handles {
            Some(handles) => handles,
            None => {
                let handles = setup(tx)?;
                self.handles = Some(handles);
                handles
            }
        };
        match operation {
            InputFact::SessionStarted { pubkey, .. } => {
                self.live_sessions.insert(pubkey.clone());
            }
            InputFact::ProcessExited {
                pubkey: Some(pubkey),
                ..
            } => {
                self.live_sessions.remove(pubkey);
            }
            _ => {}
        }
        tx.set_input(handles.live_sessions, self.live_sessions.clone())?;
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
    ReplayReport::from_harness("session_watch", &first, export_trace)
}

fn run(
    script: &DataTransactionScript<InputFact>,
) -> Result<TrellisHarness<Graph<ReconcileCommand>, ReconcileCommand>, trellis_testing::ScenarioError>
{
    let mut state = ReplayState::default();
    TrellisHarness::replay_data(
        Graph::<ReconcileCommand>::new_with_command_type,
        script,
        move |operation, tx| state.apply(operation, tx),
    )
}

fn setup(tx: &mut Transaction<'_, ReconcileCommand>) -> GraphResult<Handles> {
    let scope = tx.create_scope("session-watch")?;
    let live_sessions = tx.input::<BTreeSet<String>>("live-sessions")?;
    let watched = tx.derived(
        "watched-session-set",
        DependencyList::new([live_sessions.id()])?,
        move |ctx| Ok(ctx.input(live_sessions)?.clone()),
    )?;
    let watch_set = tx.set_collection(
        "session-watch-collection",
        DependencyList::new([watched.id()])?,
        move |ctx| Ok(ctx.derived(watched)?.clone()),
    )?;
    tx.set_resource_planner(watch_set, scope, move |ctx| {
        let mut plan = ResourcePlan::new();
        for added in &ctx.diff().added {
            plan.open(
                watch_key(&added.value),
                ctx.scope(),
                ReconcileCommand::OpenSessionWatch(added.value.clone()),
            );
        }
        for removed in &ctx.diff().removed {
            plan.close(watch_key(&removed.value), ctx.scope());
        }
        Ok(plan)
    })?;
    Ok(Handles { live_sessions })
}
