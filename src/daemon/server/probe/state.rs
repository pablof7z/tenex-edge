//! `probe state <surface>` (§4.3): live values for a surface, under its lock.
//! `subscriptions` lists each live REQ with its owner scopes + refcount;
//! `status` lists each session's currently-published content. `hook_context`
//! reports the honest not-a-live-graph note.

use super::{not_live_note, required_str, DaemonState};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn state_value(state: &Arc<DaemonState>, params: &Value) -> Result<Value> {
    let surface = required_str(params, "surface")?;
    match surface {
        "status" => {
            let r = state.status.lock().expect("status mutex poisoned");
            let rows: Vec<Value> = r
                .state_rows()
                .into_iter()
                .map(|row| {
                    json!({
                        "session": row.session,
                        "title": row.title,
                        "activity": row.activity,
                        "busy": row.busy,
                        "channels": row.channels,
                    })
                })
                .collect();
            Ok(json!({ "verb": "state", "surface": "status", "rows": rows }))
        }
        "subscriptions" => {
            let r = state.subs.lock().expect("subs mutex poisoned");
            let rows: Vec<Value> = r
                .state_rows()
                .into_iter()
                .map(|row| {
                    json!({
                        "resource_key": row.resource_key,
                        "refcount": row.refcount,
                        "owners": row.owners,
                    })
                })
                .collect();
            Ok(json!({ "verb": "state", "surface": "subscriptions", "rows": rows }))
        }
        "hook_context" => Ok(json!({ "verb": "state", "surface": "hook_context",
            "rows": [], "note": not_live_note() })),
        other => Err(anyhow::anyhow!("probe state: unknown surface `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use crate::reconcile::{CoverageSnapshot, StatusReconciler, SubscriptionReconciler};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn status_state_rows_carry_published_content() {
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
        let rows = r.state_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session, "s1");
        assert_eq!(rows[0].activity, "reading");
        assert!(rows[0].busy);
    }

    #[test]
    fn subscription_state_rows_carry_refcounts() {
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
        let rows = r.state_rows();
        // #h + #d for the channel, each owned by daemon + session scope.
        let h = rows
            .iter()
            .find(|r| r.resource_key == "sub/h/general")
            .unwrap();
        assert_eq!(h.refcount, 2);
        assert_eq!(h.owners.len(), 2);
    }
}
