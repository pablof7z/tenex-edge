//! `probe oracle` (§4.6): run the incremental-equals-full check on each
//! daemon-held reconciler graph, live, under its lock. This proves the graph's
//! own bookkeeping is self-consistent — NOT that the host effects it drove are
//! correct — so the render is scrupulously honest about what was and was not
//! proven, and names the uncovered (imperative) surfaces.

use super::{not_live_note, DaemonState};
use serde_json::{json, Value};
use std::sync::Arc;

/// Surfaces whose correctness the oracle covers (they are live, daemon-held graphs).
const COVERED: [&str; 2] = ["status", "subscriptions"];
/// Imperative surfaces with no live graph the oracle can check (frontier §2 table).
const UNCOVERED: [&str; 4] = ["turn_lifecycle", "cursor", "session_start", "outbox"];

pub(super) fn oracle_value(state: &Arc<DaemonState>) -> Value {
    let mut surfaces = Vec::new();
    let mut ok = true;

    {
        let r = state.status.lock().expect("status mutex poisoned");
        let (row, green) = check(
            "status",
            r.assert_oracle(),
            r.revision(),
            r.graph_node_count(),
        );
        ok &= green;
        surfaces.push(row);
    }
    {
        let r = state.subs.lock().expect("subs mutex poisoned");
        let (row, green) = check(
            "subscriptions",
            r.assert_oracle(),
            r.revision(),
            r.graph_node_count(),
        );
        ok &= green;
        surfaces.push(row);
    }
    surfaces.push(not_live_note());

    json!({
        "verb": "oracle",
        "ok": ok,
        "surfaces": surfaces,
        // The load-bearing honesty (§4.6, §8): a green oracle is graph-bookkeeping
        // correctness, not host-effect correctness.
        "surface_correctness_proven": false,
        "covered": COVERED,
        "uncovered": UNCOVERED,
    })
}

/// Build one surface row from its oracle outcome; returns `(row, is_green)`.
fn check(
    surface: &str,
    outcome: trellis_core::GraphResult<()>,
    revision: u64,
    nodes: usize,
) -> (Value, bool) {
    match outcome {
        Ok(()) => (
            json!({
                "surface": surface,
                "live_graph": true,
                "status": "green",
                "revision": revision,
                "nodes": nodes,
            }),
            true,
        ),
        Err(e) => (
            json!({
                "surface": surface,
                "live_graph": true,
                "status": "red",
                "revision": revision,
                "nodes": nodes,
                "error": format!("{e}"),
            }),
            false,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A freshly-driven daemon reconciler reports green for both live surfaces and
    /// prints the honest uncovered list. Built directly (no live daemon) by driving
    /// the same reconcilers the daemon holds.
    #[test]
    fn freshly_driven_reconcilers_report_green() {
        use crate::reconcile::{CoverageSnapshot, StatusReconciler, SubscriptionReconciler};
        use std::collections::{BTreeMap, BTreeSet};

        let mut status = StatusReconciler::new(90, 30);
        status
            .on_session_started(
                "s1",
                "laptop",
                "coder",
                "pk1",
                ".",
                BTreeSet::from(["room".to_string()]),
                true,
                "T",
                "reading",
                100,
            )
            .unwrap();

        let mut subs = SubscriptionReconciler::new().unwrap();
        let mut sessions = BTreeMap::new();
        sessions.insert("s1".to_string(), BTreeSet::from(["room".to_string()]));
        subs.sync(&CoverageSnapshot {
            daemon_channels: BTreeSet::from(["room".to_string()]),
            addressed_pubkeys: BTreeSet::new(),
            archived_channels: BTreeSet::new(),
            sessions,
        })
        .unwrap();

        // Mirror what oracle_value does per surface.
        let (status_row, s_green) = check(
            "status",
            status.assert_oracle(),
            status.revision(),
            status.graph_node_count(),
        );
        let (subs_row, u_green) = check(
            "subscriptions",
            subs.assert_oracle(),
            subs.revision(),
            subs.graph_node_count(),
        );

        assert!(s_green && u_green, "both live surfaces green");
        assert_eq!(status_row["status"], "green");
        assert_eq!(subs_row["status"], "green");
        assert!(status_row["nodes"].as_i64().unwrap() > 0);
    }
}
