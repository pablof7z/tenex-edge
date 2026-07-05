//! `probe acid`: verify a live `why` cause with preview counterfactuals.

use super::artifact;
use super::{required_str, DaemonState};
use crate::reconcile::journal::{InputFact, StatusDrive};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn acid_value(state: &Arc<DaemonState>, params: &Value) -> Result<Value> {
    let handle = required_str(params, "handle")?;
    let fact = artifact::fact_param(params, "fact")?.context("probe acid: requires `fact`")?;
    let causes = why_causes(state, handle)?;
    let cause = params
        .get("cause")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| causes.first().cloned())
        .context("probe acid: live why returned no input causes")?;

    let original = artifact::preview_artifact(state, &fact)?;
    let removed_fact = remove_cause(state, fact.clone(), &cause)?;
    let removed = artifact::preview_artifact(state, &removed_fact)?;
    let unrelated_fact = mutate_unrelated(fact)?;
    let unrelated = artifact::preview_artifact(state, &unrelated_fact)?;

    let removal_diff = artifact::field_diff(&original.value, &removed.value);
    let unrelated_diff = artifact::field_diff(&original.value, &unrelated.value);
    let necessary = original.hash != removed.hash;
    let unrelated_stable = original.hash == unrelated.hash;

    Ok(json!({
        "verb": "acid",
        "handle": handle,
        "surface": original.surface,
        "cause": cause,
        "input_causes": causes,
        "necessary": necessary,
        "unrelated_stable": unrelated_stable,
        "ok": necessary && unrelated_stable,
        "original_hash": original.hash,
        "removed_hash": removed.hash,
        "unrelated_hash": unrelated.hash,
        "removal_diff": removal_diff,
        "unrelated_diff": unrelated_diff,
    }))
}

fn why_causes(state: &Arc<DaemonState>, handle: &str) -> Result<Vec<String>> {
    if let Some(session) = handle.strip_prefix("status:") {
        let r = state.status.lock().expect("status mutex poisoned");
        return Ok(r
            .explain_status(session)
            .map(|why| why.input_causes)
            .unwrap_or_default());
    }
    if let Some(channel) = handle.strip_prefix("sub:") {
        let r = state.subs.lock().expect("subs mutex poisoned");
        return Ok(r.explain_channel(channel).input_causes);
    }
    Err(anyhow::anyhow!(
        "probe acid: handle must be `status:<session>` or `sub:<channel>`"
    ))
}

fn remove_cause(state: &Arc<DaemonState>, fact: InputFact, cause: &str) -> Result<InputFact> {
    match fact {
        InputFact::StatusDrive(StatusDrive::DistillCompleted {
            session_id,
            mut title,
            mut activity,
            window_hash,
            at,
        }) => {
            let current = state
                .status
                .lock()
                .expect("status mutex poisoned")
                .state_rows()
                .into_iter()
                .find(|row| row.session == session_id)
                .with_context(|| format!("probe acid: no status row for `{session_id}`"))?;
            if cause.ends_with("/activity") {
                activity = current.activity;
            } else if cause.ends_with("/title") {
                title = current.title;
            } else {
                anyhow::bail!("probe acid: unsupported status cause `{cause}`");
            }
            Ok(InputFact::StatusDrive(StatusDrive::DistillCompleted {
                session_id,
                title,
                activity,
                window_hash,
                at,
            }))
        }
        InputFact::SubscriptionSync { mut snapshot, at } => {
            if let Some(session) = subscription_session_cause(cause) {
                snapshot.sessions.remove(&session);
            } else if cause == "subscriptions/daemon/channels" {
                snapshot.daemon_channels.clear();
            } else if cause == "subscriptions/daemon/addressed_pubkeys" {
                snapshot.addressed_pubkeys.clear();
            } else if cause == "subscriptions/daemon/archived_channels" {
                snapshot.archived_channels.clear();
            } else {
                anyhow::bail!("probe acid: unsupported subscription cause `{cause}`");
            }
            Ok(InputFact::SubscriptionSync { snapshot, at })
        }
        _ => Err(anyhow::anyhow!(
            "probe acid: fact/cause combination is not supported"
        )),
    }
}

fn mutate_unrelated(fact: InputFact) -> Result<InputFact> {
    match fact {
        InputFact::StatusDrive(StatusDrive::DistillCompleted {
            session_id,
            title,
            activity,
            window_hash,
            at,
        }) => Ok(InputFact::StatusDrive(StatusDrive::DistillCompleted {
            session_id,
            title,
            activity,
            window_hash: Some(format!(
                "{}:acid-unrelated",
                window_hash.unwrap_or_else(|| "sha256".into())
            )),
            at,
        })),
        InputFact::SubscriptionSync { snapshot, at } => Ok(InputFact::SubscriptionSync {
            snapshot,
            at: at.saturating_add(999_999),
        }),
        _ => Err(anyhow::anyhow!(
            "probe acid: no unrelated mutation for this fact"
        )),
    }
}

fn subscription_session_cause(cause: &str) -> Option<String> {
    let rest = cause.strip_prefix("subscriptions/session/")?;
    let (session, field) = rest.split_once('/')?;
    (field == "channels").then(|| session.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::CoverageSnapshot;
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test]
    async fn status_acid_verifies_activity_cause_and_unrelated_hash() {
        let state = DaemonState::new_for_test().await;
        {
            let mut r = state.status.lock().unwrap();
            r.on_session_started(
                "s1",
                "host",
                "agent",
                "pk",
                ".",
                BTreeSet::from(["room".to_string()]),
                true,
                "T",
                "reading",
                100,
            )
            .unwrap();
            r.on_distill("s1", "T", "reviewing", 130).unwrap();
        }
        let fact = InputFact::StatusDrive(StatusDrive::DistillCompleted {
            session_id: "s1".into(),
            title: "T".into(),
            activity: "writing".into(),
            window_hash: Some("sha256:w2".into()),
            at: 160,
        });
        let v = acid_value(
            &state,
            &json!({ "verb": "acid", "handle": "status:s1", "fact": fact }),
        )
        .unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["necessary"], true);
        assert_eq!(v["unrelated_stable"], true);
    }

    #[tokio::test]
    async fn subscription_acid_verifies_session_channel_cause() {
        let state = DaemonState::new_for_test().await;
        let mut sessions = BTreeMap::new();
        sessions.insert("s1".to_string(), BTreeSet::from(["room".to_string()]));
        let snapshot = CoverageSnapshot {
            daemon_channels: BTreeSet::new(),
            addressed_pubkeys: BTreeSet::new(),
            archived_channels: BTreeSet::new(),
            sessions: sessions.clone(),
        };
        state.subs.lock().unwrap().sync(&snapshot).unwrap();

        sessions.insert("s2".to_string(), BTreeSet::from(["room2".to_string()]));
        let fact = InputFact::SubscriptionSync {
            snapshot: CoverageSnapshot {
                daemon_channels: BTreeSet::new(),
                addressed_pubkeys: BTreeSet::new(),
                archived_channels: BTreeSet::new(),
                sessions,
            },
            at: 200,
        };
        let v = acid_value(
            &state,
            &json!({
                "verb": "acid",
                "handle": "sub:room",
                "cause": "subscriptions/session/s1/channels",
                "fact": fact,
            }),
        )
        .unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["necessary"], true);
        assert_eq!(v["unrelated_stable"], true);
    }
}
