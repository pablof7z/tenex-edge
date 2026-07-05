//! `probe why <handle>` (§4.3): live causality for one handle, under the
//! reconciler lock, from the dependency-path audit already computed on the last
//! commit. Handles: `sub:<channel>` (the REQ's owners + refcount + last command
//! cause) and `status:<session>` (the last status command + its input causes).
//! Everything is rendered through the label registry. When no live audit exists
//! for a handle, that is reported cleanly rather than faked.

use super::{required_str, DaemonState};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn why_value(state: &Arc<DaemonState>, params: &Value) -> Result<Value> {
    let handle = required_str(params, "handle")?;

    if let Some(channel) = handle.strip_prefix("sub:") {
        let r = state.subs.lock().expect("subs mutex poisoned");
        let why = r.explain_channel(channel);
        return Ok(json!({
            "verb": "why",
            "handle": handle,
            "kind": "subscription",
            "resource_key": why.resource_key,
            "refcount": why.refcount,
            "owners": why.owners,
            "last_kind": why.last_kind,
            "cause": why.cause,
            "found": why.last_kind.is_some(),
        }));
    }

    if let Some(session) = handle.strip_prefix("status:") {
        let r = state.status.lock().expect("status mutex poisoned");
        return Ok(match r.explain_status(session) {
            Some(why) => json!({
                "verb": "why",
                "handle": handle,
                "kind": "status",
                "resource_key": why.resource_key,
                "last_kind": why.last_kind,
                "cause": why.cause,
                "input_causes": why.input_causes,
                "found": true,
            }),
            None => json!({
                "verb": "why",
                "handle": handle,
                "kind": "status",
                "found": false,
                "note": "no command emitted yet on this daemon graph",
            }),
        });
    }

    Err(anyhow::anyhow!(
        "probe why: handle must be `sub:<channel>` or `status:<session>`"
    ))
}

#[cfg(test)]
mod tests {
    use crate::reconcile::{CoverageSnapshot, StatusReconciler, SubscriptionReconciler};
    use std::collections::{BTreeMap, BTreeSet};

    /// `status:` explain surfaces the labeled last command + its input cause.
    #[test]
    fn status_handle_explains_last_command() {
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
            "",
            100,
        )
        .unwrap();
        r.on_distill("s1", "T", "compiling", 100).unwrap();

        let why = r.explain_status("s1").unwrap();
        assert_eq!(why.resource_key, "status/s1");
        assert_eq!(why.last_kind, "Replace");
        assert!(why.input_causes.iter().any(|l| l == "status/s1/activity"));
    }

    /// `sub:` explain surfaces owners + refcount + the labeled cause.
    #[test]
    fn sub_handle_explains_owners_and_refcount() {
        let mut r = SubscriptionReconciler::new().unwrap();
        let mut sessions = BTreeMap::new();
        sessions.insert("s1".to_string(), BTreeSet::from(["general".to_string()]));
        r.sync(&CoverageSnapshot {
            daemon_channels: BTreeSet::from(["general".to_string()]),
            addressed_pubkeys: BTreeSet::new(),
            archived_channels: BTreeSet::new(),
            sessions,
        })
        .unwrap();

        let why = r.explain_channel("general");
        assert_eq!(why.resource_key, "sub/h/general");
        assert_eq!(why.refcount, 2);
        assert_eq!(why.last_kind.as_deref(), Some("Open"));
    }
}
