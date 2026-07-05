//! Probe-facing, non-mutating queries over the live status graph (frontier
//! design §3 keystone + §4.3 why). Two capabilities, both read-only:
//!
//! * [`StatusReconciler::preview_on_distill`] stages a distill fact and calls
//!   `tx.preview()` — the Terraform-plan for a status change. It runs the full
//!   commit pipeline on the transaction's private working clone and returns the
//!   would-be [`TransactionResult`] WITHOUT the swap, so the real graph (state,
//!   revision, audit) is left exactly as it was. This is the "never gambles on
//!   what it will do" half of the North Star made observable.
//! * [`StatusReconciler::explain_status`] answers "why did this session's status
//!   last (re)publish" from the dependency-path audit already computed under
//!   `opts()`, rendered through the label registry — the live-causality `why`.

use trellis_core::{GraphResult, ResourceCommandCause, ScopeId, Transaction, TransactionResult};

use crate::reconcile::labels::key_path;

use super::model::{opts, status_key, SessionNodes};
use super::{StatusCommand, StatusReconciler};

/// One session's live status values: the derived, currently-published content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusStateRow {
    pub session: String,
    pub title: String,
    pub activity: String,
    pub busy: bool,
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
    /// Canonical input facts that caused it, as labels (e.g. `status/s1/activity`).
    pub input_causes: Vec<String>,
}

impl StatusReconciler {
    /// The live graph revision — identical before and after a [`preview`](Self::preview_on_distill),
    /// which is exactly the "simulation applies nothing" guarantee.
    pub fn revision(&self) -> u64 {
        self.graph.revision().get()
    }

    /// Dry-run a distill fact: stage `title`/`activity` (both optional) and, when
    /// `now` is given, re-arm the TTL exactly as [`on_distill`](Self::on_distill)
    /// would — then `preview()` instead of `commit()`. The real graph is never
    /// mutated. An unknown session previews an empty transaction so callers always
    /// get a receipt. Omitting `now` stages ONLY the content fact, so the plan is
    /// a content-exact dedup: identical content ⇒ zero commands.
    pub fn preview_on_distill(
        &mut self,
        id: &str,
        title: Option<&str>,
        activity: Option<&str>,
        now: Option<u64>,
    ) -> GraphResult<TransactionResult<StatusCommand>> {
        self.preview_stage(id, now, |tx, _n| {
            if let Some(t) = title {
                tx.set_input(_n.title, t.to_string())?;
            }
            if let Some(a) = activity {
                tx.set_input(_n.activity, a.to_string())?;
            }
            Ok(())
        })
    }

    /// Shared preview helper mirroring [`mutate`](Self::mutate) but ending in
    /// `tx.preview()`. `&mut self` is required only to begin the transaction; the
    /// transaction is consumed by `preview` and mutates nothing.
    fn preview_stage(
        &mut self,
        id: &str,
        now: Option<u64>,
        stage: impl FnOnce(&mut Transaction<'_, StatusCommand>, &SessionNodes) -> GraphResult<()>,
    ) -> GraphResult<TransactionResult<StatusCommand>> {
        let Some(nodes) = self.sessions.get(id).copied() else {
            let tx = self.graph.begin_transaction_with_options(opts())?;
            return tx.preview();
        };
        let mut tx = self.graph.begin_transaction_with_options(opts())?;
        stage(&mut tx, &nodes)?;
        if let Some(n) = now {
            tx.set_input(nodes.arm, n / self.refresh_secs)?;
        }
        tx.preview()
    }

    /// Live per-session status values — the derived content currently published,
    /// in session-id order. Sourced from the last-published shadow (every started
    /// session has an opening publish), so it is the exact wire content.
    pub fn state_rows(&self) -> Vec<StatusStateRow> {
        self.last
            .values()
            .map(|cmd| StatusStateRow {
                session: cmd.session_id.clone(),
                title: cmd.title.clone(),
                activity: cmd.activity.clone(),
                busy: cmd.busy,
                channels: cmd.channels.clone(),
            })
            .collect()
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

/// Does this preview's plan carry any status effect? A `Close` becomes the
/// final expiring status publish at the host seam, so any command means publish.
pub fn would_publish(result: &TransactionResult<StatusCommand>) -> bool {
    !result.resource_plan.commands().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Seed a live, busy session with one channel and assert the opening publish.
    fn seeded_busy(
        id: &str,
        title: &str,
        activity: &str,
        channels: &[&str],
        now: u64,
    ) -> StatusReconciler {
        let mut r = StatusReconciler::new(90, 30);
        let chans: BTreeSet<String> = channels.iter().map(|s| s.to_string()).collect();
        let out = r
            .on_session_started(
                id, "laptop", "coder", "pk1", ".", chans, true, title, activity, now,
            )
            .unwrap();
        assert_eq!(out.effects.len(), 1, "startup opens");
        r.assert_oracle().unwrap();
        r
    }

    /// THE keystone guarantee: `preview_on_distill` applies NOTHING. The revision
    /// is unchanged, the oracle stays green, and the live input still reads its
    /// pre-sim value — proven three ways, including via the dedup itself.
    #[test]
    fn preview_applies_nothing_to_the_live_graph() {
        let mut r = seeded_busy("s1", "T", "reading", &["room"], 100);
        let rev0 = r.revision();

        // Simulate a NEW activity: the plan WOULD publish a Replace and names the
        // activity input as changed — but the graph is untouched.
        let plan = r
            .preview_on_distill("s1", None, Some("reviewing the PR"), None)
            .unwrap();
        assert!(would_publish(&plan), "new activity would publish");
        let changed = r.labels().labels_for(&plan.changed_inputs);
        assert!(
            changed.iter().any(|l| l == "status/s1/activity"),
            "changed inputs name the activity: {changed:?}"
        );

        // (1) revision identical, (2) oracle still green.
        assert_eq!(r.revision(), rev0, "preview must not bump the revision");
        r.assert_oracle().unwrap();

        // (3) the live input still reads its pre-sim value: previewing the ORIGINAL
        // activity now dedups to an empty plan — impossible if the earlier preview
        // had mutated the graph to "reviewing the PR".
        let dedup = r
            .preview_on_distill("s1", None, Some("reading"), None)
            .unwrap();
        assert!(
            !would_publish(&dedup) && dedup.resource_plan.commands().is_empty(),
            "unchanged content dedups to no plan: {:?}",
            dedup.resource_plan.commands()
        );

        // And a REAL distill of the new activity still publishes — the graph never
        // silently absorbed the previews.
        let real = r.on_distill("s1", "T", "reviewing the PR", 100).unwrap();
        assert_eq!(
            real.effects.len(),
            1,
            "the real distill publishes: {:?}",
            real.effects
        );
        assert_eq!(r.revision(), rev0 + 1, "exactly one real commit landed");
        r.assert_oracle().unwrap();
    }

    /// `explain_status` reports the latest command labeled through the registry.
    #[test]
    fn explain_status_labels_the_latest_command() {
        let mut r = seeded_busy("s1", "T", "", &["room"], 100);
        r.on_distill("s1", "T", "compiling", 100).unwrap();

        let why = r.explain_status("s1").expect("a command was emitted");
        assert_eq!(why.resource_key, "status/s1");
        assert_eq!(why.last_kind, "Replace");
        assert!(why.cause.starts_with("planner: status/s1"));
        assert!(
            why.input_causes.iter().any(|l| l == "status/s1/activity"),
            "attributed to the activity input: {:?}",
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
