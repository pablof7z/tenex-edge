//! `probe simulate status` (§3, the keystone): dry-run a distill fact against the
//! live status graph via `tx.preview()` and report the would-be plan — the
//! Terraform-plan for a status change. NOTHING is applied: the preview runs the
//! full commit pipeline on the transaction's private working clone and is
//! discarded, so the daemon's graph (state, revision, audit) is untouched. The
//! plan is returned as LABELED JSON: the resource commands, the changed inputs,
//! and a `would_publish` flag. Identical content dedups to an empty plan — the
//! live dedup, demonstrated without a relay round-trip.

use super::{required_str, DaemonState};
use crate::reconcile::labels::key_path;
use crate::reconcile::status::probe::would_publish;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use trellis_core::ResourceCommand;

/// kind:30315 — the NIP status event a status Open/Replace/Refresh publishes.
const STATUS_KIND: u64 = 30315;

pub(super) fn simulate_value(state: &Arc<DaemonState>, params: &Value) -> Result<Value> {
    let surface = params
        .get("surface")
        .and_then(Value::as_str)
        .unwrap_or("status");
    if surface == "subscriptions" {
        return Ok(json!({
            "verb": "simulate", "surface": "subscriptions",
            "implemented": false,
            "message": "subscriptions simulate is a v2 follow-up; status simulate is live",
        }));
    }
    if surface != "status" {
        return Err(anyhow::anyhow!(
            "probe simulate: unknown surface `{surface}`"
        ));
    }

    let session = required_str(params, "session")?;
    let title = params.get("title").and_then(Value::as_str);
    let activity = params.get("activity").and_then(Value::as_str);
    let now = params.get("now").and_then(Value::as_u64);

    let mut r = state.status.lock().expect("status mutex poisoned");
    let rev_before = r.revision();
    let plan = r.preview_on_distill(session, title, activity, now)?;
    let rev_after = r.revision();

    let commands: Vec<Value> = plan
        .resource_plan
        .commands()
        .iter()
        .map(|c| {
            json!({
                "op": op_str(c),
                "resource": key_path(c.key()),
                "kind": STATUS_KIND,
                "publish": is_publish(c),
            })
        })
        .collect();
    let changed = r.labels().labels_for(&plan.changed_inputs);

    Ok(json!({
        "verb": "simulate",
        "surface": "status",
        "session": session,
        "fact": { "kind": "distill", "title": title, "activity": activity },
        "would_publish": would_publish(&plan),
        "commands": commands,
        "changed": changed,
        "revision_before": rev_before,
        "revision_after": rev_after,
        "ok": true,
    }))
}

fn op_str<C>(c: &ResourceCommand<C>) -> &'static str {
    match c {
        ResourceCommand::Open { .. } => "Open",
        ResourceCommand::Replace { .. } => "Replace",
        ResourceCommand::Refresh { .. } => "Refresh",
        ResourceCommand::Close { .. } => "Close",
    }
}

fn is_publish<C>(c: &ResourceCommand<C>) -> bool {
    !matches!(c, ResourceCommand::Close { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::StatusReconciler;
    use std::collections::BTreeSet;

    fn seeded() -> StatusReconciler {
        let mut r = StatusReconciler::new(90, 30);
        r.on_session_started(
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
        r
    }

    /// A changed activity previews a publishing Replace and leaves the revision put.
    #[test]
    fn changed_activity_would_publish() {
        let mut r = seeded();
        let rev = r.revision();
        let plan = r
            .preview_on_distill("s1", None, Some("reviewing the PR"), None)
            .unwrap();
        assert!(would_publish(&plan));
        assert_eq!(r.revision(), rev, "preview did not bump revision");
        let cmd = &plan.resource_plan.commands()[0];
        assert_eq!(op_str(cmd), "Replace");
        assert_eq!(key_path(cmd.key()), "status/s1");
        assert!(is_publish(cmd));
    }

    /// Identical content dedups to an empty plan — NO CHANGE.
    #[test]
    fn identical_content_is_no_change() {
        let mut r = seeded();
        let plan = r
            .preview_on_distill("s1", None, Some("reading"), None)
            .unwrap();
        assert!(!would_publish(&plan));
        assert!(plan.resource_plan.commands().is_empty());
    }

    /// `--now` is unix **SECONDS**, not millis (regression for the CLI help that
    /// wrongly said "wall-clock millis"). The TTL re-arm bucket is `now /
    /// refresh_secs`; a `now` in the SAME bucket as the seed must NOT fabricate a
    /// TTL refresh into the previewed plan. Proven three ways: (1) a genuine
    /// content change with no `now` plans exactly the content Replace; (2) the
    /// same change with a same-bucket seconds `now` plans the SAME single Replace
    /// — no extra Refresh, `arm` never appears among the changed inputs; (3) a
    /// pure dedup (unchanged content) with a same-bucket seconds `now` stays an
    /// empty plan, whereas feeding the SAME instant misread as MILLIS jumps the
    /// bucket and fabricates the spurious Refresh — the exact bug the wrong unit
    /// invited. The unit itself is pinned by the bucket assertions.
    #[test]
    fn now_is_unix_seconds_and_same_bucket_adds_no_spurious_refresh() {
        // refresh_secs = 30; seed exactly on a bucket boundary so a few seconds
        // later is unambiguously the SAME bucket.
        const REFRESH_SECS: u64 = 30;
        let now_seed: u64 = 1_700_000_010; // a plausible unix-SECONDS stamp, % 30 == 0
        let now_same_bucket: u64 = now_seed + 5; // still the same TTL bucket
        assert_eq!(
            now_seed / REFRESH_SECS,
            now_same_bucket / REFRESH_SECS,
            "seconds a few apart share a TTL bucket — the pinned unit semantics",
        );
        // The same wall-clock instant misread as MILLIS lands in a DIFFERENT
        // bucket: this is precisely why the unit must be seconds.
        assert_ne!(
            now_seed / REFRESH_SECS,
            (now_seed * 1000) / REFRESH_SECS,
            "millis and seconds map to different buckets — the unit is load-bearing",
        );

        let mut r = StatusReconciler::new(90, REFRESH_SECS);
        r.on_session_started(
            "s1",
            "laptop",
            "coder",
            "pk1",
            ".",
            BTreeSet::from(["room".to_string()]),
            true,
            "T",
            "reading",
            now_seed,
        )
        .unwrap();

        // (1) genuine content change, NO `now`: exactly the content Replace.
        let no_now = r
            .preview_on_distill("s1", None, Some("reviewing the PR"), None)
            .unwrap();
        assert!(would_publish(&no_now), "content change would publish");
        assert_eq!(no_now.resource_plan.commands().len(), 1);
        assert_eq!(op_str(&no_now.resource_plan.commands()[0]), "Replace");
        let changed = r.labels().labels_for(&no_now.changed_inputs);
        assert!(
            changed.iter().any(|l| l == "status/s1/activity"),
            "the plan is about the activity change: {changed:?}"
        );

        // (2) same content change WITH a same-bucket seconds `now`: still the one
        // content Replace — no fabricated TTL Refresh, `arm` never changed.
        let with_now = r
            .preview_on_distill("s1", None, Some("reviewing the PR"), Some(now_same_bucket))
            .unwrap();
        assert!(would_publish(&with_now));
        assert_eq!(
            with_now.resource_plan.commands().len(),
            1,
            "same-bucket now adds NO extra command: {:?}",
            with_now.resource_plan.commands()
        );
        assert_eq!(op_str(&with_now.resource_plan.commands()[0]), "Replace");
        let changed_with_now = r.labels().labels_for(&with_now.changed_inputs);
        assert!(
            !changed_with_now.iter().any(|l| l == "status/s1/arm"),
            "no spurious TTL re-arm in the plan: {changed_with_now:?}"
        );

        // (3) pure dedup (unchanged content): a same-bucket seconds `now` keeps the
        // plan empty; the same instant misread as MILLIS fabricates a Refresh.
        let dedup = r
            .preview_on_distill("s1", None, Some("reading"), Some(now_same_bucket))
            .unwrap();
        assert!(
            dedup.resource_plan.commands().is_empty(),
            "same-bucket seconds `now` fabricates nothing: {:?}",
            dedup.resource_plan.commands()
        );
        let millis = r
            .preview_on_distill("s1", None, Some("reading"), Some(now_seed * 1000))
            .unwrap();
        assert_eq!(
            millis.resource_plan.commands().len(),
            1,
            "millis jumps the bucket and fabricates a spurious re-arm"
        );
        assert_eq!(
            op_str(&millis.resource_plan.commands()[0]),
            "Refresh",
            "the fabricated command is a pure TTL Refresh — the bug the wrong unit invited"
        );
    }
}
