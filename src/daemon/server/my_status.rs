use super::channel_membership_rpc::resolve_caller;
use super::*;

#[derive(serde::Deserialize)]
struct MyStatusParams {
    topic: String,
}

pub(in crate::daemon::server) fn rpc_my_status(
    state: &Arc<DaemonState>,
    params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let p: MyStatusParams = serde_json::from_value(params.clone()).context("my status params")?;
    let topic = crate::work_topic::normalize(&p.topic)?;
    let rec = resolve_caller(state, params, "my status")?;
    let set_at = now_secs();
    state.with_store(|s| s.set_session_work_topic(&rec.session_id, &topic, set_at))?;
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

    #[tokio::test]
    async fn stores_topic_for_the_exact_caller_session() {
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

        let response = rpc_my_status(
            &state,
            &serde_json::json!({
                "pty_session": "pty-1",
                "topic": "Researching MCP improvements around resource allocation",
            }),
        )
        .unwrap();

        assert_eq!(response["session_id"], session_id);
        let rec = state.with_store(|s| s.get_session(&session_id).unwrap().unwrap());
        assert_eq!(
            rec.work_topic,
            "Researching MCP improvements around resource allocation"
        );
        assert!(rec.work_topic_set_at > 0);
    }

    #[tokio::test]
    async fn rejects_topics_over_fifteen_words() {
        let state = DaemonState::new_for_test().await;
        let topic = vec!["word"; crate::work_topic::MAX_WORDS + 1].join(" ");
        let err = rpc_my_status(&state, &serde_json::json!({ "topic": topic })).unwrap_err();
        assert!(err.to_string().contains("at most"));
    }
}
