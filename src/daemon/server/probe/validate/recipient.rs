//! Message-recipient edge validation for direct delivery/addressing claims.

use super::report::{bool_at, str_at};
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct RecipientTarget {
    message_prefix: String,
    recipient_pubkey: String,
    target_session: Option<String>,
}

pub(super) fn recipient_target(target: &str) -> Option<RecipientTarget> {
    colon_target(target, "recipient:")
        .or_else(|| colon_target(target, "delivery:"))
        .or_else(|| path_target(target, "recipient/"))
        .or_else(|| path_target(target, "delivery/"))
}

pub(super) fn recipient_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &RecipientTarget,
) -> Value {
    let result = state.with_store(|store| {
        let Some(message) = store.get_message_by_prefix(&parsed.message_prefix)? else {
            return Ok::<_, anyhow::Error>((None, Vec::new(), None, None, None, None));
        };
        let recipients = store.message_recipients(&message.message_id)?;
        let target_session = match parsed.target_session.as_deref() {
            Some(raw) => store
                .get_session(raw)?
                .map(|session| session.session_id)
                .or_else(|| Some(raw.to_string())),
            None => None,
        };
        let profile = store.get_profile(&parsed.recipient_pubkey)?;
        let identity = store.get_identity(&parsed.recipient_pubkey)?;
        let bound_session = match identity.as_ref().filter(|row| !row.session_id.is_empty()) {
            Some(row) => store.get_session(&row.session_id)?,
            None => None,
        };
        Ok::<_, anyhow::Error>((
            Some(message),
            recipients,
            target_session,
            profile,
            identity,
            bound_session,
        ))
    });
    let (message, recipients, resolved_target, profile, identity, bound_session) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "recipient",
                "message_prefix": parsed.message_prefix,
                "recipient_pubkey": parsed.recipient_pubkey,
                "target_session": parsed.target_session,
                "supported": true,
                "found": false,
                "error": e.to_string(),
                "summary": "recipient evidence could not read durable message ledgers",
                "reason": e.to_string(),
            });
        }
    };
    let Some(message) = message else {
        return json!({
            "target": target,
            "kind": "recipient",
            "message_prefix": parsed.message_prefix,
            "recipient_pubkey": parsed.recipient_pubkey,
            "target_session": parsed.target_session,
            "target_session_resolved": resolved_target,
            "supported": true,
            "message_found": false,
            "found": false,
            "summary": format!(
                "message `{}` is not in the canonical chat read model",
                parsed.message_prefix
            ),
            "reason": "no messages row matched this local message id or native event id prefix",
        });
    };

    let matching_rows = recipients
        .iter()
        .filter(|row| row.recipient_pubkey == parsed.recipient_pubkey)
        .filter(|row| {
            resolved_target
                .as_deref()
                .is_none_or(|target| row.target_session.as_deref().unwrap_or_default() == target)
        })
        .collect::<Vec<_>>();
    let pubkey_row_count = recipients
        .iter()
        .filter(|row| row.recipient_pubkey == parsed.recipient_pubkey)
        .count();
    let delivered = matching_rows.iter().any(|row| row.delivered_at.is_some());
    let pending = !matching_rows.is_empty() && !delivered;
    let failed_sync = is_failed_state(&message.sync_state)
        || message
            .error
            .as_deref()
            .is_some_and(|error| !error.is_empty());
    let summary = summary(
        &message.message_id,
        &parsed.recipient_pubkey,
        resolved_target.as_deref(),
        !matching_rows.is_empty(),
        delivered,
        pending,
        failed_sync,
        recipients.len(),
    );
    let reason = reason(
        !matching_rows.is_empty(),
        delivered,
        pending,
        failed_sync,
        recipients.len(),
        pubkey_row_count,
        resolved_target.is_some(),
    );

    json!({
        "target": target,
        "kind": "recipient",
        "message_prefix": parsed.message_prefix,
        "message_id": message.message_id,
        "recipient_pubkey": parsed.recipient_pubkey,
        "target_session": parsed.target_session,
        "target_session_resolved": resolved_target,
        "supported": true,
        "message_found": true,
        "found": !matching_rows.is_empty(),
        "delivered": delivered,
        "pending": pending,
        "message_sync_state": message.sync_state,
        "message_error": message.error,
        "message_channel_h": message.channel_h,
        "message_native_event_id": message.native_event_id,
        "recipient_count": recipients.len(),
        "pubkey_row_count": pubkey_row_count,
        "matching_row_count": matching_rows.len(),
        "rows": matching_rows.iter().take(8).map(|row| json!({
            "message_id": row.message_id,
            "recipient_pubkey": row.recipient_pubkey,
            "target_session": row.target_session,
            "delivered_at": row.delivered_at,
        })).collect::<Vec<_>>(),
        "profile_found": profile.is_some(),
        "profile_slug": profile.as_ref().map(|p| p.slug.as_str()).unwrap_or(""),
        "profile_name": profile.as_ref().map(|p| p.name.as_str()).unwrap_or(""),
        "identity_found": identity.is_some(),
        "identity_alive": identity.as_ref().is_some_and(|i| i.alive),
        "identity_session_id": identity.as_ref().map(|i| i.session_id.as_str()).unwrap_or(""),
        "bound_session_found": bound_session.is_some(),
        "bound_session_alive": bound_session.as_ref().is_some_and(|s| s.alive),
        "summary": summary,
        "reason": reason,
    })
}

pub(super) fn push_recipient_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let recipient_count = evidence
        .get("recipient_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let pubkey_row_count = evidence
        .get("pubkey_row_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let session_mismatch = !bool_at(evidence, "found")
        && bool_at(evidence, "message_found")
        && !str_at(evidence, "target_session").is_empty()
        && pubkey_row_count > 0;
    let status = if !str_at(evidence, "error").is_empty()
        || !str_at(evidence, "message_error").is_empty()
        || is_failed_state(str_at(evidence, "message_sync_state"))
        || (!bool_at(evidence, "found")
            && bool_at(evidence, "message_found")
            && recipient_count > 0
            && !session_mismatch)
    {
        "failed"
    } else if bool_at(evidence, "delivered") {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "recipient",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    } else if status == "passed" && !bool_at(evidence, "profile_found") {
        limitations.push(
            "recipient edge is delivered, but no relay profile is cached for the pubkey".into(),
        );
    }
}

fn colon_target(target: &str, prefix: &str) -> Option<RecipientTarget> {
    let rest = target.strip_prefix(prefix)?;
    let mut parts = rest.splitn(3, ':');
    let message_prefix = parts.next()?;
    let recipient_pubkey = parts.next()?;
    let target_session = parts.next();
    build_target(message_prefix, recipient_pubkey, target_session)
}

fn path_target(target: &str, prefix: &str) -> Option<RecipientTarget> {
    let rest = target.strip_prefix(prefix)?;
    let mut parts = rest.splitn(3, '/');
    let message_prefix = parts.next()?;
    let recipient_pubkey = parts.next()?;
    let target_session = parts.next();
    build_target(message_prefix, recipient_pubkey, target_session)
}

fn build_target(
    message_prefix: &str,
    recipient_pubkey: &str,
    target_session: Option<&str>,
) -> Option<RecipientTarget> {
    (!message_prefix.trim().is_empty() && !recipient_pubkey.trim().is_empty()).then(|| {
        RecipientTarget {
            message_prefix: message_prefix.to_string(),
            recipient_pubkey: recipient_pubkey.to_string(),
            target_session: target_session
                .filter(|session| !session.trim().is_empty())
                .map(str::to_string),
        }
    })
}

fn summary(
    message_id: &str,
    pubkey: &str,
    target_session: Option<&str>,
    found: bool,
    delivered: bool,
    pending: bool,
    failed_sync: bool,
    recipient_count: usize,
) -> String {
    let suffix = target_session
        .map(|session| format!(" session `{session}`"))
        .unwrap_or_default();
    if failed_sync {
        return format!(
            "message `{message_id}` failed before recipient `{pubkey}` could be proven"
        );
    }
    if delivered {
        return format!("message `{message_id}` was delivered to recipient `{pubkey}`{suffix}");
    }
    if pending {
        return format!(
            "message `{message_id}` addresses recipient `{pubkey}`{suffix}, delivery pending"
        );
    }
    if found {
        return format!("message `{message_id}` has recipient `{pubkey}`{suffix}");
    }
    if recipient_count > 0 {
        format!("message `{message_id}` does not address recipient `{pubkey}`{suffix}")
    } else {
        format!("message `{message_id}` has no durable recipient edges")
    }
}

fn reason(
    found: bool,
    delivered: bool,
    pending: bool,
    failed_sync: bool,
    recipient_count: usize,
    pubkey_row_count: usize,
    target_session_requested: bool,
) -> &'static str {
    if failed_sync {
        return "message row records a failed/rejected sync state";
    }
    if delivered {
        return "message_recipients contains a delivered edge for this recipient";
    }
    if pending {
        return "message_recipients contains the recipient edge, but delivered_at is not set";
    }
    if found {
        return "message_recipients contains the recipient edge";
    }
    if target_session_requested && pubkey_row_count > 0 {
        return "recipient pubkey is present, but not for the requested target session";
    }
    if recipient_count > 0 {
        return "message has hydrated recipient edges and this pubkey is absent";
    }
    "recipient edges are not hydrated for this message"
}

fn is_failed_state(state: &str) -> bool {
    matches!(state, "failed" | "error" | "rejected")
}
