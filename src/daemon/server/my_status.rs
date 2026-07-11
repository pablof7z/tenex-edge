use super::channel_membership_rpc::resolve_caller;
use super::*;
use crate::replay_capsules::status_fact;

#[derive(serde::Deserialize)]
struct MyStatusParams {
    topic: String,
}

pub(in crate::daemon::server) async fn rpc_my_status(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: MyStatusParams = serde_json::from_value(params.clone()).context("my status params")?;
    let topic = crate::work_topic::normalize(&p.topic)?;
    let rec = resolve_caller(state, params, "my status")?;
    let set_at = now_secs();
    state.with_store(|s| s.set_session_work_topic(&rec.session_id, &topic, set_at))?;
    let keys = state.session_signing_keys(&rec.session_id)?;
    crate::status_seam::drive(
        &state.status,
        state.fabric_provider(),
        &keys,
        &state.store,
        &state.outbox,
        crate::status_seam::DriveMeta {
            trigger: "manual_title",
            window_hash: None,
            replay_fact: Some(status_fact!(title, rec.session_id, topic, set_at)),
        },
        |r| r.on_title_set(&rec.session_id, &topic, set_at),
    )
    .await;
    state.outbox_notify.notify_waiters();
    Ok(serde_json::json!({
        "session_id": rec.session_id,
        "topic": topic,
        "distill_paused_until": set_at.saturating_add(crate::work_topic::DISTILL_PAUSE_SECS),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::RegisterSession;
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn stores_and_publishes_title_for_the_exact_caller_session() {
        let state = DaemonState::new_for_test().await;
        let session_id = state.with_store(|s| {
            s.register_session(&RegisterSession {
                harness: "codex".into(),
                external_id_kind: "pty_session".into(),
                external_id: "pty-1".into(),
                agent_pubkey: "pk".into(),
                agent_slug: "codex".into(),
                channel_h: "root".into(),
                child_pid: None,
                transcript_path: None,
                resume_id: String::new(),
                now: 1,
            })
            .unwrap()
        });
        {
            let mut status = state.status.lock().unwrap();
            let out = status
                .on_session_started(
                    &session_id,
                    "test-host",
                    "codex",
                    "pk",
                    ".",
                    BTreeSet::from(["root".to_string()]),
                    true,
                    "",
                    "checking logs",
                    1,
                )
                .unwrap();
            assert_eq!(out.effects.len(), 1);
        }

        let response = rpc_my_status(
            &state,
            &serde_json::json!({
                "pty_session": "pty-1",
                "topic": "Researching MCP improvements around resource allocation",
            }),
        )
        .await
        .unwrap();

        assert_eq!(response["session_id"], session_id);
        let rec = state.with_store(|s| s.get_session(&session_id).unwrap().unwrap());
        assert_eq!(
            rec.work_topic,
            "Researching MCP improvements around resource allocation"
        );
        assert!(rec.work_topic_set_at > 0);
        assert_eq!(
            rec.title,
            "Researching MCP improvements around resource allocation"
        );

        let rows = state.with_store(|s| s.peek_outbox(10, u64::MAX).unwrap());
        assert_eq!(rows.len(), 1);
        let event: serde_json::Value = serde_json::from_str(&rows[0].event_json).unwrap();
        assert_eq!(event["kind"].as_i64(), Some(30315));
        assert!(event["tags"].as_array().unwrap().iter().any(|tag| {
            tag.as_array().is_some_and(|parts| {
                parts.first().and_then(|v| v.as_str()) == Some("title")
                    && parts.get(1).and_then(|v| v.as_str())
                        == Some("Researching MCP improvements around resource allocation")
            })
        }));
    }

    #[tokio::test]
    async fn rejects_topics_over_fifteen_words() {
        let state = DaemonState::new_for_test().await;
        let topic = vec!["word"; crate::work_topic::MAX_WORDS + 1].join(" ");
        let err = rpc_my_status(&state, &serde_json::json!({ "topic": topic }))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("at most"));
    }
}
