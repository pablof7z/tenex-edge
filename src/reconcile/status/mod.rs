//! Per-session kind:30315 status reconciler — the ONE authority that decides
//! WHEN a session's public status is (re)published.
//! One graph owns dedup, refresh, h-tags, and deterministic teardown; the host
//! only signs and enqueues the emitted effects.
mod command;
mod model;
mod preview;
pub(crate) mod probe;
pub(crate) mod replay;
mod revoke;
mod status_build;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use trellis_core::{
    Graph, GraphResult, NodeId, ResourceCommand, ResourceCommandExplanation, Transaction,
    TransactionResult,
};

use crate::domain::Status;
use crate::reconcile::labels::NodeLabels;

pub use command::{StatusCommand, StatusOutcome};
use model::{create_session, opts, status_key, SessionNodes, StaticInfo};

/// Why the reconciler is asking the host to publish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishReason {
    /// First status for a session.
    Opened,
    /// The derived content changed (turn edge, title, channel change).
    Changed,
    /// Content unchanged; only the NIP-40 TTL window was re-armed.
    Refreshed,
}

/// A host effect: the exact status to sign + enqueue on the outbox.
#[derive(Clone, Debug, PartialEq)]
pub enum StatusEffect {
    /// Publish this status (fresh TTL) for the given reason.
    Publish {
        status: Status,
        reason: PublishReason,
    },
    /// Explicit relay retraction for callers that need immediate disappearance.
    /// Normal session end does not use this; it publishes idle with a full TTL.
    Expire { status: Status },
}

#[derive(Clone)]
pub struct StatusReconciler {
    graph: Graph<StatusCommand>,
    ttl_secs: u64,
    refresh_secs: u64,
    sessions: BTreeMap<String, SessionNodes>,
    last: BTreeMap<String, StatusCommand>,
    labels: NodeLabels,
}

impl StatusReconciler {
    /// Build an empty reconciler. `ttl_secs` is the NIP-40 window; `refresh_secs`
    /// is the re-arm cadence (`on_tick` re-arms once per `refresh_secs` bucket).
    pub fn new(ttl_secs: u64, refresh_secs: u64) -> Self {
        Self {
            graph: Graph::<StatusCommand>::new_with_command_type(),
            ttl_secs: ttl_secs.max(1),
            refresh_secs: refresh_secs.max(1),
            sessions: BTreeMap::new(),
            last: BTreeMap::new(),
            labels: NodeLabels::new(),
        }
    }

    pub fn labels(&self) -> &NodeLabels {
        &self.labels
    }

    pub fn graph_node_count(&self) -> usize {
        self.graph.nodes().count()
    }

    /// Daemon constructor: TTL from a `Duration`, cadence from the domain heartbeat.
    pub fn for_ttl(ttl: Duration) -> Self {
        Self::new(ttl.as_secs(), crate::domain::HEARTBEAT_SECS)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_session_started(
        &mut self,
        pubkey: &str,
        host: &str,
        slug: &str,
        rel_cwd: &str,
        channels: BTreeSet<String>,
        working: bool,
        automatic_delivery: bool,
        title: &str,
        now: u64,
    ) -> GraphResult<StatusOutcome> {
        self.on_session_started_with_dispatch(
            pubkey,
            host,
            slug,
            rel_cwd,
            channels,
            working,
            automatic_delivery,
            title,
            None,
            now,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_session_started_with_dispatch(
        &mut self,
        pubkey: &str,
        host: &str,
        slug: &str,
        rel_cwd: &str,
        channels: BTreeSet<String>,
        working: bool,
        automatic_delivery: bool,
        title: &str,
        dispatch_event: Option<String>,
        now: u64,
    ) -> GraphResult<StatusOutcome> {
        if self.sessions.contains_key(pubkey) {
            return self.empty_commit();
        }
        let info = StaticInfo {
            host: host.to_string(),
            slug: slug.to_string(),
            rel_cwd: rel_cwd.to_string(),
            dispatch_event,
        };
        let (nodes, result) = create_session(
            &mut self.graph,
            &mut self.labels,
            pubkey,
            info,
            channels,
            working,
            automatic_delivery,
            title,
            now / self.refresh_secs,
        )?;
        self.sessions.insert(pubkey.to_string(), nodes);
        let effects = self.translate(&result, now);
        Ok(StatusOutcome { effects, result })
    }

    /// A turn started (working) / ended (idle).
    pub fn on_turn_start(&mut self, id: &str, now: u64) -> GraphResult<StatusOutcome> {
        self.mutate(id, now, |tx, n| tx.set_input(n.working, true))
    }

    pub fn on_turn_end(&mut self, id: &str, now: u64) -> GraphResult<StatusOutcome> {
        self.mutate(id, now, |tx, n| tx.set_input(n.working, false))
    }

    /// A manual title was declared by the session owner.
    pub fn on_title_set(&mut self, id: &str, title: &str, now: u64) -> GraphResult<StatusOutcome> {
        self.mutate(id, now, |tx, n| tx.set_input(n.title, title.to_string()))
    }

    pub fn on_channels_changed(
        &mut self,
        id: &str,
        channels: BTreeSet<String>,
        now: u64,
    ) -> GraphResult<StatusOutcome> {
        self.mutate(id, now, move |tx, n| tx.set_input(n.channels, channels))
    }

    /// A clock tick: re-arm the NIP-40 window (a `Refresh`, not a content change)
    /// if `now` crossed a refresh bucket; otherwise nothing.
    pub fn on_tick(
        &mut self,
        id: &str,
        automatic_delivery: bool,
        now: u64,
    ) -> GraphResult<StatusOutcome> {
        self.mutate(id, now, |tx, n| {
            tx.set_input(n.automatic_delivery, automatic_delivery)
        })
    }

    /// The session ended: publish one immediately-expiring offline status.
    pub fn on_session_ended(&mut self, id: &str, now: u64) -> GraphResult<StatusOutcome> {
        let refresh_secs = self.refresh_secs;
        let final_arm = end_arm(now, refresh_secs);
        self.mutate(id, now, |tx, n| {
            tx.set_input(n.live, false)?;
            tx.set_input(n.working, false)?;
            tx.set_input(n.arm, final_arm)
        })
    }

    /// Drop a stale ended session from the local status graph without publishing a
    /// relay retraction. Its last idle status naturally expires by NIP-40.
    pub fn forget_session(&mut self, id: &str) -> GraphResult<()> {
        let Some(nodes) = self.sessions.remove(id) else {
            return Ok(());
        };
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        tx.close_scope(nodes.scope)?;
        tx.commit()?;
        self.last.remove(id);
        Ok(())
    }

    pub fn why_command(&self, id: &str) -> Option<&ResourceCommandExplanation> {
        self.graph.why_resource_command(&status_key(id))
    }

    pub fn title_input(&self, id: &str) -> Option<NodeId> {
        self.sessions.get(id).map(|nodes| nodes.title.id())
    }

    pub fn assert_oracle(&self) -> GraphResult<()> {
        self.graph.assert_incremental_equals_full()?;
        Ok(())
    }

    /// Stage the caller's input, re-sync the TTL arm bucket, commit, translate.
    fn mutate(
        &mut self,
        id: &str,
        now: u64,
        stage: impl FnOnce(&mut Transaction<'_, StatusCommand>, &SessionNodes) -> GraphResult<()>,
    ) -> GraphResult<StatusOutcome> {
        let Some(nodes) = self.sessions.get(id).copied() else {
            return self.empty_commit();
        };
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        tx.set_input(nodes.arm, now / self.refresh_secs)?;
        stage(&mut tx, &nodes)?;
        let result = tx.commit()?;
        drop(tx);
        let effects = self.translate(&result, now);
        Ok(StatusOutcome { effects, result })
    }

    /// Empty commit for an unknown session, so callers always get a receipt.
    fn empty_commit(&mut self) -> GraphResult<StatusOutcome> {
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        let result = tx.commit()?;
        drop(tx);
        Ok(StatusOutcome {
            effects: Vec::new(),
            result,
        })
    }

    /// Turn the graph's resource plan into host effects, maintaining the
    /// last-published shadow used to build closing/expiring publishes.
    fn translate(
        &mut self,
        result: &TransactionResult<StatusCommand>,
        now: u64,
    ) -> Vec<StatusEffect> {
        let mut effects = Vec::new();
        for command in result.resource_plan.commands() {
            let (cmd, reason) = match command {
                ResourceCommand::Open { command, .. } => (command, PublishReason::Opened),
                ResourceCommand::Replace { command, .. } => (command, PublishReason::Changed),
                ResourceCommand::Refresh { command, .. } => (command, PublishReason::Refreshed),
                ResourceCommand::Close { key, .. } => {
                    if let Some(cmd) = key.segment(1).and_then(|sid| self.last.remove(sid)) {
                        // Keep the last-known `h` tags so the retraction lands.
                        effects.push(StatusEffect::Expire {
                            status: self.to_status(&cmd, now, true),
                        });
                    }
                    continue;
                }
            };
            self.last.insert(cmd.pubkey.clone(), cmd.clone());
            effects.push(StatusEffect::Publish {
                status: self.to_status(cmd, now, false),
                reason,
            });
        }
        effects
    }

    fn to_status(&self, cmd: &StatusCommand, now: u64, expiring: bool) -> Status {
        status_build::to_status(cmd, self.ttl_secs, now, expiring)
    }
}

fn end_arm(now: u64, refresh_secs: u64) -> u64 {
    now / refresh_secs.max(1) + 1
}
