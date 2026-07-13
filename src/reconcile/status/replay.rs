use std::collections::BTreeMap;

use anyhow::Result;
use trellis_core::{Graph, GraphResult, Transaction};
use trellis_testing::{DataTransactionScript, TrellisHarness};

use crate::reconcile::journal::{InputFact, StatusDrive, StatusSessionStartedArgs};
use crate::reconcile::labels::NodeLabels;
use crate::reconcile::replay::ReplayReport;

use super::model::{stage_session, SessionNodes, StaticInfo};
use super::preview::status_drive_from_fact;
use super::{end_arm, StatusCommand, StatusReconciler};

const STATUS_REFRESH_SECS: u64 = crate::domain::HEARTBEAT_SECS;

impl StatusReconciler {
    /// Build the graph seed needed to replay a later per-session drive.
    pub fn replay_seed(&self, id: &str) -> Option<StatusSessionStartedArgs> {
        let nodes = self.sessions.get(id)?;
        let last = self.last.get(id)?;
        Some(StatusSessionStartedArgs {
            pubkey: id.to_string(),
            host: last.host.clone(),
            slug: last.slug.clone(),
            rel_cwd: last.rel_cwd.clone(),
            dispatch_event: last.dispatch_event.clone(),
            channels: self
                .graph
                .input_value(nodes.channels)
                .ok()
                .flatten()?
                .clone(),
            working: *self.graph.input_value(nodes.working).ok().flatten()?,
            automatic_delivery: *self
                .graph
                .input_value(nodes.automatic_delivery)
                .ok()
                .flatten()?,
            title: self.graph.input_value(nodes.title).ok().flatten()?.clone(),
            activity: self
                .graph
                .input_value(nodes.activity)
                .ok()
                .flatten()?
                .clone(),
            at: self
                .graph
                .input_value(nodes.arm)
                .ok()
                .flatten()?
                .saturating_mul(self.refresh_secs),
        })
    }
}

struct ReplayState {
    sessions: BTreeMap<String, SessionNodes>,
    labels: NodeLabels,
    refresh_secs: u64,
}

impl ReplayState {
    fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
            labels: NodeLabels::new(),
            refresh_secs: STATUS_REFRESH_SECS.max(1),
        }
    }

    fn apply(
        &mut self,
        operation: &InputFact,
        tx: &mut Transaction<'_, StatusCommand>,
    ) -> GraphResult<()> {
        let Some(drive) = status_drive_from_fact(operation) else {
            return Ok(());
        };
        match &drive {
            StatusDrive::SessionStarted(args) => {
                if self.sessions.contains_key(&args.pubkey) {
                    return Ok(());
                }
                let info = StaticInfo {
                    host: args.host.clone(),
                    slug: args.slug.clone(),
                    rel_cwd: args.rel_cwd.clone(),
                    dispatch_event: args.dispatch_event.clone(),
                };
                let nodes = stage_session(
                    tx,
                    &mut self.labels,
                    &args.pubkey,
                    info,
                    args.channels.clone(),
                    args.working,
                    args.automatic_delivery,
                    &args.title,
                    &args.activity,
                    args.at / self.refresh_secs,
                )?;
                self.sessions.insert(args.pubkey.clone(), nodes);
            }
            StatusDrive::TurnStarted { pubkey, at } => {
                self.mutate(pubkey, tx, *at, |tx, n| tx.set_input(n.working, true))?;
            }
            StatusDrive::TurnEnded { pubkey, at } => {
                self.mutate(pubkey, tx, *at, |tx, n| tx.set_input(n.working, false))?;
            }
            StatusDrive::DistillCompleted {
                pubkey,
                title,
                activity,
                at,
                ..
            } => {
                self.mutate(pubkey, tx, *at, |tx, n| {
                    tx.set_input(n.title, title.clone())?;
                    tx.set_input(n.activity, activity.clone())
                })?;
            }
            StatusDrive::TitleSet { pubkey, title, at } => {
                self.mutate(pubkey, tx, *at, |tx, n| {
                    tx.set_input(n.title, title.clone())
                })?;
            }
            StatusDrive::ChannelsChanged {
                pubkey,
                channels,
                at,
            } => {
                self.mutate(pubkey, tx, *at, |tx, n| {
                    tx.set_input(n.channels, channels.clone())
                })?;
            }
            StatusDrive::Tick {
                pubkey,
                automatic_delivery,
                at,
            } => {
                self.mutate(pubkey, tx, *at, |tx, n| {
                    tx.set_input(n.automatic_delivery, *automatic_delivery)
                })?;
            }
            StatusDrive::SessionEnded { pubkey, at } => {
                if let Some(nodes) = self.sessions.get(pubkey) {
                    tx.set_input(nodes.live, false)?;
                    tx.set_input(nodes.working, false)?;
                    tx.set_input(nodes.arm, end_arm(*at, self.refresh_secs))?;
                }
            }
            StatusDrive::SessionRevoked { pubkey, .. } => {
                if let Some(nodes) = self.sessions.remove(pubkey) {
                    tx.close_scope(nodes.scope)?;
                }
            }
        }
        Ok(())
    }

    fn mutate(
        &self,
        pubkey: &str,
        tx: &mut Transaction<'_, StatusCommand>,
        at: u64,
        stage: impl FnOnce(&mut Transaction<'_, StatusCommand>, &SessionNodes) -> GraphResult<()>,
    ) -> GraphResult<()> {
        let Some(nodes) = self.sessions.get(pubkey) else {
            return Ok(());
        };
        stage(tx, nodes)?;
        tx.set_input(nodes.arm, at / self.refresh_secs)
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
    ReplayReport::from_harness("status", &first, export_trace)
}

fn run(
    script: &DataTransactionScript<InputFact>,
) -> Result<TrellisHarness<Graph<StatusCommand>, StatusCommand>, trellis_testing::ScenarioError> {
    let mut state = ReplayState::new();
    TrellisHarness::replay_data(
        Graph::<StatusCommand>::new_with_command_type,
        script,
        move |operation, tx| state.apply(operation, tx),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::reconcile::{InputFact, StatusSessionStartedArgs};

    use super::*;

    #[test]
    fn status_capsule_replays_independently() {
        let mut script = DataTransactionScript::new();
        script
            .step("start")
            .operation(InputFact::StatusDrive(StatusDrive::SessionStarted(
                StatusSessionStartedArgs {
                    pubkey: "pk1".into(),
                    host: "laptop".into(),
                    slug: "coder".into(),
                    rel_cwd: ".".into(),
                    channels: BTreeSet::from(["room".to_string()]),
                    working: true,
                    title: "T".into(),
                    activity: "reading".into(),
                    dispatch_event: None,
                    at: 100,
                },
            )))
            .commit();
        script
            .step("distill")
            .operation(InputFact::StatusDrive(StatusDrive::DistillCompleted {
                pubkey: "pk1".into(),
                title: "T".into(),
                activity: "reviewing".into(),
                window_hash: Some("sha256:abc".into()),
                at: 130,
            }))
            .commit();

        let report = replay_script(&script, true).unwrap();
        assert_eq!(report.surface, "status");
        assert_eq!(report.steps, 2);
        assert!(report.resource_commands >= 2);
        assert!(report.trace_json.is_some());
    }
}
