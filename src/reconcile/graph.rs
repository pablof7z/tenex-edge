//! A thin [`Reconciler`] wrapping a `trellis_core::Graph`.
//!
//! This is the minimal, REAL proof-of-life for Trellis in tenex-edge. It wires
//! one input, one derived node, one set-collection, and one resource planner
//! that emits open/close commands for a trivial [`ResourceKey`], all committed
//! through a transaction that returns a `TransactionResult` receipt.
//!
//! The reconciled fact modeled here is deliberately trivial — the *set of live
//! sessions the daemon is watching* — folded from [`InputFact`]s. It exists to
//! prove the pattern compiles against the real API and that the receipt exposes
//! `why_resource_command` / `why_changed` and the full-recompute oracle. The
//! surface reconcilers replace this body; the shape stays.

use std::collections::BTreeSet;

use trellis_core::{
    DependencyList, Graph, GraphResult, InputNode, ResourceCommandExplanation, ResourceKey,
    ResourcePlan, TransactionResult,
};

use super::journal::InputFact;
use super::labels::NodeLabels;

mod probe;
pub(crate) mod replay;

pub use probe::{SessionWatchPreview, SessionWatchStateRow, SessionWatchWhy};

/// Host-defined command payload emitted by the reconciler's planners.
///
/// Trellis returns these as plain data inside the resource plan; the host
/// applies them. No I/O ever happens inside the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconcileCommand {
    /// Begin watching liveness for a session.
    OpenSessionWatch(String),
}

/// Reconciler spine: canonical [`InputFact`]s in, resource plans + receipts out.
#[derive(Clone)]
pub struct Reconciler {
    graph: Graph<ReconcileCommand>,
    /// Canonical input: the set of session ids the world says are live.
    live_sessions: InputNode<BTreeSet<String>>,
    labels: NodeLabels,
}

impl Reconciler {
    /// Builds the graph: input -> derived -> set-collection -> planner.
    pub fn new() -> GraphResult<Self> {
        let mut graph = Graph::<ReconcileCommand>::new_with_command_type();
        let mut tx = graph.begin_transaction()?;

        // A scope owns the reconciled resources' lifecycle.
        let scope = tx.create_scope("session-watch")?;

        // Canonical input: live session ids handed to us by the host.
        let live_sessions = tx.input::<BTreeSet<String>>("live-sessions")?;
        let mut labels = NodeLabels::new();
        labels.record(live_sessions.id(), "session_watch/live_sessions");
        tx.set_input(live_sessions, BTreeSet::new())?;

        // Derived node: the watched-session set (identity here, but a real node
        // on the dependency path so audit/oracle have something to explain).
        let watched = tx.derived(
            "watched-session-set",
            DependencyList::new([live_sessions.id()])?,
            move |ctx| Ok(ctx.input(live_sessions)?.clone()),
        )?;
        labels.record(watched.id(), "session_watch/watched_sessions");

        // Set-collection: structurally diffed against the previous commit.
        let watch_set = tx.set_collection(
            "session-watch-collection",
            DependencyList::new([watched.id()])?,
            move |ctx| Ok(ctx.derived(watched)?.clone()),
        )?;
        labels.record(watch_set.id(), "session_watch/resources");

        // Resource planner: turn the diff into open/close commands (data only).
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

        tx.commit()?;
        drop(tx);

        Ok(Self {
            graph,
            live_sessions,
            labels,
        })
    }

    /// Folds one canonical fact into the graph and returns the receipt.
    ///
    /// Only session-lifecycle facts move the watched set here; every other fact
    /// still commits (yielding an empty plan), which is exactly what the spine
    /// needs to prove: facts flow in, decisions come out, always as data.
    pub fn apply(&mut self, fact: &InputFact) -> GraphResult<TransactionResult<ReconcileCommand>> {
        let live = self.next_live_sessions(fact);
        let mut tx = self.graph.begin_transaction()?;
        tx.set_input(self.live_sessions, live)?;
        let result = tx.commit()?;
        drop(tx);
        Ok(result)
    }

    /// Reads the current live-session input value.
    fn current_live_sessions(&self) -> BTreeSet<String> {
        self.graph
            .input_value(self.live_sessions)
            .ok()
            .flatten()
            .cloned()
            .unwrap_or_default()
    }

    fn next_live_sessions(&self, fact: &InputFact) -> BTreeSet<String> {
        let mut live = self.current_live_sessions();
        match fact {
            InputFact::SessionStarted { pubkey, .. } => {
                live.insert(pubkey.clone());
            }
            InputFact::ProcessExited {
                pubkey: Some(pubkey),
                ..
            } => {
                live.remove(pubkey);
            }
            _ => {}
        }
        live
    }

    /// Audit query: why the latest command for a resource key was emitted.
    pub fn why_command(&self, key: &ResourceKey) -> Option<&ResourceCommandExplanation> {
        self.graph.why_resource_command(key)
    }

    /// The full-recompute oracle: incremental state must equal a rebuild.
    pub fn assert_oracle(&self) -> GraphResult<()> {
        self.graph.assert_incremental_equals_full()?;
        Ok(())
    }
}

/// Resource identity for one watched session.
pub fn watch_key(pubkey: &str) -> ResourceKey {
    ResourceKey::from_segments(["session-watch", pubkey])
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_core::ResourceCommand;
    use trellis_testing::ResourceLedger;

    fn started(id: &str, at: u64) -> InputFact {
        InputFact::SessionStarted {
            pubkey: id.to_owned(),
            channel_h: None,
            pid: None,
            at,
        }
    }

    fn exited(id: &str, at: u64) -> InputFact {
        InputFact::ProcessExited {
            pubkey: Some(id.to_owned()),
            pid: 1,
            at,
        }
    }

    #[test]
    fn drives_transactions_and_oracle_agrees() {
        let mut r = Reconciler::new().unwrap();
        let mut ledger = ResourceLedger::new();

        // Open two watches.
        let open = r.apply(&started("s1", 10)).unwrap();
        ledger.apply_result(&open);
        r.assert_oracle().unwrap();
        assert!(open.resource_plan.commands().iter().any(|c| matches!(
            c,
            ResourceCommand::Open { key, command, .. }
                if key == &watch_key("s1")
                    && command == &ReconcileCommand::OpenSessionWatch("s1".to_owned())
        )));

        let open2 = r.apply(&started("s2", 11)).unwrap();
        ledger.apply_result(&open2);
        r.assert_oracle().unwrap();

        // A non-lifecycle fact still commits, emitting no commands.
        let tick = r.apply(&InputFact::ClockTick { at: 12 }).unwrap();
        assert!(tick.resource_plan.commands().is_empty());
        r.assert_oracle().unwrap();

        // Close one watch; the diff must produce exactly its close command.
        let close = r.apply(&exited("s1", 13)).unwrap();
        ledger.apply_result(&close);
        assert!(close.resource_plan.commands().iter().any(|c| matches!(
            c,
            ResourceCommand::Close { key, .. } if key == &watch_key("s1")
        )));
        r.assert_oracle().unwrap();

        // The Trellis testing ledger confirms no orphaned resources remain.
        ledger.assert_resource_not_open(&watch_key("s1")).unwrap();
        ledger.assert_all_resources_have_owner().unwrap();
    }

    #[test]
    fn audit_explains_emitted_command() {
        let mut r = Reconciler::new().unwrap();
        r.apply(&started("s1", 10)).unwrap();

        let explanation = r
            .why_command(&watch_key("s1"))
            .expect("an open command was emitted for s1");
        assert_eq!(explanation.key, watch_key("s1"));
        r.assert_oracle().unwrap();
    }
}
