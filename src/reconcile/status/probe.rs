//! Probe-facing, non-mutating queries over the live status graph (frontier
//! design §3 keystone + §4.3 why). [`StatusReconciler::explain_status`] answers
//! "why did this session's status
//!   last (re)publish" from the dependency-path audit already computed under
//!   `opts()`, rendered through the label registry — the live-causality `why`.

use trellis_core::{ResourceCommandCause, ScopeId, TransactionResult};

use crate::reconcile::labels::key_path;

use super::model::status_key;
use super::{StatusCommand, StatusReconciler};

/// One session's live status values: the derived, currently-published content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusStateRow {
    pub session: String,
    pub title: String,
    pub state: crate::session_state::SessionState,
    pub channels: Vec<String>,
}

/// Plain, Trellis-free explanation of a session status's latest command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusWhy {
    /// Human resource path, e.g. `status/s1`.
    pub resource_key: String,
    /// The latest command operation (`Open`/`Replace`/`Refresh`/`Close`).
    pub last_kind: String,
    /// What produced it: `planner: <collection-label>` or `scope-closed: <scope>`.
    pub cause: String,
    /// Canonical input facts that caused it, as labels (e.g. `status/s1/title`).
    pub input_causes: Vec<String>,
}

impl StatusReconciler {
    /// The live graph revision.
    pub fn revision(&self) -> u64 {
        self.graph.revision().get()
    }

    /// Live per-session status values — the derived content currently published,
    /// in session-id order. Sourced from the last-published shadow (every started
    /// session has an opening publish), so it is the exact wire content.
    pub fn state_rows(&self) -> Vec<StatusStateRow> {
        self.last
            .values()
            .map(|cmd| StatusStateRow {
                session: cmd.pubkey.clone(),
                title: cmd.title.clone(),
                state: cmd.state,
                channels: cmd.channels.clone(),
            })
            .collect()
    }

    /// Current TTL arm converted back to the timestamp bucket boundary.
    pub(crate) fn current_arm_at(&self, id: &str) -> Option<u64> {
        let nodes = self.sessions.get(id)?;
        self.graph
            .input_value(nodes.arm)
            .ok()
            .flatten()
            .map(|arm| arm.saturating_mul(self.refresh_secs))
    }

    /// Explain the latest command emitted for a session's status, resolved through
    /// the label registry. `None` when no command has ever been emitted for `id`
    /// on this daemon graph (no live audit to report — say so, don't fake it).
    pub fn explain_status(&self, id: &str) -> Option<StatusWhy> {
        let why = self.why_command(id)?;
        Some(StatusWhy {
            resource_key: key_path(&status_key(id)),
            last_kind: format!("{:?}", why.kind),
            cause: self.cause_label(&why.cause),
            input_causes: self.labels().labels_for(&why.input_causes),
        })
    }

    /// Render a resource command cause with labels: `Planner` names the collection
    /// it consumed; `ScopeClosed` names the torn-down scope.
    fn cause_label(&self, cause: &ResourceCommandCause) -> String {
        match cause {
            ResourceCommandCause::Planner { collection } => format!(
                "planner: {}",
                self.labels()
                    .label_of(*collection)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("node:{}", collection.get()))
            ),
            ResourceCommandCause::ScopeClosed { scope } => {
                format!("scope-closed: {}", self.scope_label(*scope))
            }
        }
    }

    /// The debug name of a scope, or `scope:<n>` if unknown.
    fn scope_label(&self, scope: ScopeId) -> String {
        self.graph
            .scope_meta(scope)
            .map(|m| m.debug_name().to_string())
            .unwrap_or_else(|| format!("scope:{}", scope.get()))
    }
}

/// Does this preview's plan carry any status effect? Session end is modeled as a
/// final idle publish with normal TTL, so any status command means publish.
pub fn would_publish(result: &TransactionResult<StatusCommand>) -> bool {
    !result.resource_plan.commands().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::StatusDrive;
    use std::collections::BTreeSet;

    /// Seed a live, busy session with one channel and assert the opening publish.
    fn seeded_busy(id: &str, title: &str, channels: &[&str], now: u64) -> StatusReconciler {
        let mut r = StatusReconciler::new(90, 30);
        let chans: BTreeSet<String> = channels.iter().map(|s| s.to_string()).collect();
        let out = r
            .on_session_started(id, "laptop", "coder", ".", chans, true, true, title, now)
            .unwrap();
        assert_eq!(out.effects.len(), 1, "startup opens");
        r.assert_oracle().unwrap();
        r
    }

    /// Previewing an agent title update applies nothing to the live graph.
    #[test]
    fn preview_applies_nothing_to_the_live_graph() {
        let mut r = seeded_busy("s1", "T", &["room"], 100);
        let rev0 = r.revision();

        // Simulate a new title: the plan would publish a Replace, but the graph
        // stays untouched.
        let plan = r
            .preview_drive(&StatusDrive::TitleSet {
                pubkey: "s1".into(),
                title: "Reviewing the PR".into(),
                at: 100,
            })
            .unwrap()
            .result;
        assert!(would_publish(&plan), "new title would publish");
        let changed = r.labels().labels_for(&plan.changed_inputs);
        assert!(
            changed.iter().any(|l| l == "status/s1/title"),
            "changed inputs name the title: {changed:?}"
        );

        // (1) revision identical, (2) oracle still green.
        assert_eq!(r.revision(), rev0, "preview must not bump the revision");
        r.assert_oracle().unwrap();

        // Previewing the original title dedups to an empty plan, proving the
        // earlier preview did not mutate the graph.
        let dedup = r
            .preview_drive(&StatusDrive::TitleSet {
                pubkey: "s1".into(),
                title: "T".into(),
                at: 100,
            })
            .unwrap()
            .result;
        assert!(
            !would_publish(&dedup) && dedup.resource_plan.commands().is_empty(),
            "unchanged content dedups to no plan: {:?}",
            dedup.resource_plan.commands()
        );

        let real = r.on_title_set("s1", "Reviewing the PR", 100).unwrap();
        assert_eq!(
            real.effects.len(),
            1,
            "the real title update publishes: {:?}",
            real.effects
        );
        assert_eq!(r.revision(), rev0 + 1, "exactly one real commit landed");
        r.assert_oracle().unwrap();
    }

    /// `explain_status` reports the latest command labeled through the registry.
    #[test]
    fn explain_status_labels_the_latest_command() {
        let mut r = seeded_busy("s1", "T", &["room"], 100);
        r.on_title_set("s1", "Compiling", 100).unwrap();

        let why = r.explain_status("s1").expect("a command was emitted");
        assert_eq!(why.resource_key, "status/s1");
        assert_eq!(why.last_kind, "Replace");
        assert!(why.cause.starts_with("planner: status/s1"));
        assert!(
            why.input_causes.iter().any(|l| l == "status/s1/title"),
            "attributed to the title input: {:?}",
            why.input_causes
        );
    }

    /// An unknown session has no live audit — reported honestly as `None`.
    #[test]
    fn explain_status_unknown_session_is_none() {
        let r = StatusReconciler::new(90, 30);
        assert!(r.explain_status("ghost").is_none());
    }
}
