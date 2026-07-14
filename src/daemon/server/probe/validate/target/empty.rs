use serde_json::{json, Value};

pub(in crate::daemon::server::probe::validate) fn empty_handle_evidence(
    target: Option<&str>,
) -> Option<Value> {
    let target = target?;
    EMPTY_HANDLE_PREFIXES.iter().find_map(|(prefix, label)| {
        target.strip_prefix(prefix).and_then(|rest| {
            let id = rest.split('/').next().unwrap_or(rest);
            id.is_empty().then(|| {
                json!({
                    "target": target,
                    "supported": false,
                    "valid": false,
                    "kind": "empty_handle",
                    "summary": format!("target `{target}` is missing a {label}"),
                    "reason": format!("target `{target}` must include a non-empty {label} after `{prefix}`"),
                })
            })
        })
    })
}

const EMPTY_HANDLE_PREFIXES: &[(&str, &str)] = &[
    ("agent:", "agent slug"),
    ("agent/", "agent slug"),
    ("awareness:", "awareness channel"),
    ("awareness/", "awareness channel"),
    ("backend:", "backend label"),
    ("backend/", "backend label"),
    ("capsule:", "replay capsule id"),
    ("channel:", "channel id"),
    ("channel/", "channel id"),
    ("commit:", "commit ledger row id"),
    ("commit/", "commit ledger row id"),
    ("cursor:", "cursor session"),
    ("cursor/", "cursor resource"),
    ("event/", "event id"),
    ("hook:", "hook_context session"),
    ("hook/", "hook_context resource"),
    ("inbox:", "inbound event id"),
    ("inbox/", "inbound event id"),
    ("joined:", "joined session"),
    ("joined/", "joined session"),
    ("llm:", "LLM call id"),
    ("llm/", "LLM call id"),
    ("member:", "channel membership relation"),
    ("member/", "channel membership relation"),
    ("message:", "message id"),
    ("message/", "message resource"),
    ("outbox:", "outbox local id"),
    ("outbox/", "outbox resource"),
    ("planner:", "planner label"),
    ("profile:", "profile pubkey"),
    ("profile/", "profile pubkey"),
    ("pubkey:", "pubkey"),
    ("pubkey/", "pubkey"),
    ("quarantine:", "quarantined event id"),
    ("quarantine/", "quarantined event id"),
    ("receipt:", "receipt id"),
    ("recipient:", "message recipient edge"),
    ("recipient/", "message recipient edge"),
    ("session:", "session pubkey"),
    ("session/", "session resource"),
    ("session-watch:", "session_watch session"),
    ("session-watch/", "session_watch resource"),
    ("status:", "status session"),
    ("status/", "status resource"),
    ("sub:", "subscription channel"),
    ("sub/", "subscription resource"),
    ("table:", "durable table name"),
    ("table/", "durable table name"),
    ("turn:", "turn session"),
    ("turn/", "turn resource"),
    ("txn:", "transaction id"),
    ("txn/", "transaction resource"),
    ("workspace:", "channel workspace binding"),
    ("workspace/", "channel workspace binding"),
];
