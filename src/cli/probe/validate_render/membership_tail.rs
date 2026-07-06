//! Channel membership evidence renderer for `probe validate`.

use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "membership evidence");
    let _ = writeln!(
        out,
        "  - channel={} pubkey={} role={} require_admin={} found={} snapshot={}",
        str_at(evidence, "channel_h"),
        str_at(evidence, "pubkey"),
        str_at(evidence, "role"),
        bool_at(evidence, "require_admin"),
        bool_at(evidence, "found"),
        bool_at(evidence, "membership_snapshot")
    );
    let _ = writeln!(
        out,
        "  - channel_found={} members={} admins={} updated_at={}",
        bool_at(evidence, "channel_found"),
        int_at(evidence, "member_count"),
        int_at(evidence, "admin_count"),
        int_at(evidence, "updated_at")
    );
    if bool_at(evidence, "profile_found") || bool_at(evidence, "identity_found") {
        let _ = writeln!(
            out,
            "  - profile={} slug={:?} identity={} session={} alive={}",
            bool_at(evidence, "profile_found"),
            str_at(evidence, "profile_slug"),
            bool_at(evidence, "identity_found"),
            str_at(evidence, "identity_session_id"),
            bool_at(evidence, "session_alive")
        );
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

pub(super) fn render_snapshot(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "membership snapshot evidence");
    let _ = writeln!(
        out,
        "  - channel={} complete={} channel_found={} snapshot_sets={} members={} admins={}",
        str_at(evidence, "channel_h"),
        bool_at(evidence, "snapshot_complete"),
        bool_at(evidence, "channel_found"),
        int_at(evidence, "set_count"),
        int_at(evidence, "member_count"),
        int_at(evidence, "admin_count")
    );
    let _ = writeln!(
        out,
        "  - admin_set={} updated_at={} member_set={} updated_at={}",
        bool_at(evidence, "admin_set_found"),
        int_at(evidence, "admin_set_updated_at"),
        bool_at(evidence, "member_set_found"),
        int_at(evidence, "member_set_updated_at")
    );
    if let Some(members) = evidence.get("members").and_then(Value::as_array) {
        for member in members.iter().take(5) {
            let _ = writeln!(
                out,
                "  - {} role={} updated_at={}",
                str_at(member, "pubkey"),
                str_at(member, "role"),
                int_at(member, "updated_at")
            );
        }
        if members.len() > 5 {
            let _ = writeln!(out, "  - ... {} more member row(s)", members.len() - 5);
        }
    }
    if !str_at(evidence, "reason").is_empty() {
        let _ = writeln!(out, "  - {}", str_at(evidence, "reason"));
    }
}

fn str_at<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn bool_at(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn int_at(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}
