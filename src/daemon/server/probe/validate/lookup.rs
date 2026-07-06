//! Lookup validation for pasted ids, pubkeys, session ids, and other handles.

use super::report::str_at;
use super::DaemonState;
use nostr_sdk::prelude::{FromBech32, Nip19, PublicKey};
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) fn explicit_lookup_target(target: &str) -> Option<String> {
    target
        .strip_prefix("lookup:")
        .or_else(|| target.strip_prefix("lookup/"))
        .or_else(|| target.strip_prefix("find:"))
        .or_else(|| target.strip_prefix("find/"))
        .or_else(|| target.strip_prefix("id:"))
        .or_else(|| target.strip_prefix("id/"))
        .filter(|needle| !needle.trim().is_empty())
        .map(|needle| normalize_lookup_value(needle).unwrap_or_else(|| needle.trim().to_string()))
}

pub(super) fn bare_lookup_target(target: &str) -> Option<String> {
    let value = target.trim();
    if value.contains([':', '/', ' ']) || value.is_empty() {
        return normalize_lookup_value(value);
    }
    if matches!(
        value,
        "awareness" | "who" | "coverage" | "validation_coverage" | "validation-coverage"
    ) || super::super::SURFACES.contains(&value)
    {
        return None;
    }
    normalize_lookup_value(value)
        .or_else(|| (value.len() >= 8 || value.contains('-')).then(|| value.to_string()))
}

pub(super) fn lookup_evidence(state: &Arc<DaemonState>, target: &str, needle: &str) -> Value {
    let result = state.with_store(|store| super::table_samples::lookup_targets(store, needle, 20));
    let matches = match result {
        Ok(matches) => matches,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "validation_lookup",
                "needle": needle,
                "supported": true,
                "found": false,
                "ok": false,
                "error": e.to_string(),
                "summary": format!("lookup `{needle}` could not read durable state"),
                "reason": e.to_string(),
            });
        }
    };
    let found = !matches.is_empty();
    json!({
        "target": target,
        "kind": "validation_lookup",
        "needle": needle,
        "supported": true,
        "found": found,
        "ok": found,
        "match_count": matches.len(),
        "matches": matches,
        "summary": lookup_summary(needle, found),
        "reason": lookup_reason(found),
    })
}

pub(super) fn push_lookup_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    let found = evidence.get("found").and_then(Value::as_bool) == Some(true);
    let status = if !str_at(evidence, "error").is_empty() {
        "failed"
    } else if found {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "lookup",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn lookup_summary(needle: &str, found: bool) -> String {
    if found {
        format!("lookup `{needle}` matched durable validation handle(s)")
    } else {
        format!("lookup `{needle}` found no durable validation handles")
    }
}

fn lookup_reason(found: bool) -> &'static str {
    if found {
        "matches are concrete validation handles; run any target to inspect that row"
    } else {
        "no known durable table identifier column contained this value"
    }
}

fn normalize_lookup_value(value: &str) -> Option<String> {
    let raw = value.trim();
    let entity = raw.strip_prefix("nostr:").unwrap_or(raw);
    if let Ok(pk) = PublicKey::parse(entity) {
        return Some(pk.to_hex());
    }
    match Nip19::from_bech32(entity).ok()? {
        Nip19::Pubkey(pubkey) => Some(pubkey.to_hex()),
        Nip19::Profile(profile) => Some(profile.public_key.to_hex()),
        Nip19::EventId(event_id) => Some(event_id.to_hex()),
        Nip19::Event(event) => Some(event.event_id.to_hex()),
        _ => None,
    }
}
