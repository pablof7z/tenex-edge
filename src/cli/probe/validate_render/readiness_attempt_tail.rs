use serde_json::Value;
use std::fmt::Write as _;

pub(super) fn render(out: &mut String, evidence: &Value) {
    let _ = writeln!(out);
    let _ = writeln!(out, "readiness attempt evidence");
    if !bool_at(evidence, "found") {
        let _ = writeln!(out, "  - attempt {}: not found", int_at(evidence, "id"));
    } else {
        let _ = writeln!(
            out,
            "  - attempt={} channel={} outcome={} current_ready={}",
            int_at(evidence, "id"),
            str_at(evidence, "channel_h"),
            str_at(evidence, "outcome"),
            bool_at(evidence, "current_ready")
        );
        let _ = writeln!(
            out,
            "  - source={} created_at={} reason={:?}",
            str_at(evidence, "source"),
            int_at(evidence, "created_at"),
            str_at(evidence, "attempt_reason")
        );
        let _ = writeln!(
            out,
            "  - channel_found={} name={:?} members={} admins={} snapshot={}",
            bool_at(evidence, "channel_found"),
            str_at(evidence, "channel_name"),
            int_at(evidence, "member_count"),
            int_at(evidence, "admin_count"),
            bool_at(evidence, "membership_snapshot")
        );
        if !str_at(evidence, "expect_member").is_empty() {
            let _ = writeln!(
                out,
                "  - expected_member={} found={} role={:?}",
                str_at(evidence, "expect_member"),
                bool_at(evidence, "expected_member_found"),
                str_at(evidence, "expected_member_role")
            );
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
