use super::{i64_at, str_at, strs};
use serde_json::Value;
use std::fmt::Write as _;

/// `probe simulate` — the would-be plan; nothing is applied.
pub(in crate::cli::probe) fn render_simulate(v: &Value) -> String {
    let mut out = String::new();
    let surface = str_at(v, "surface");
    if v.get("simulated").and_then(Value::as_bool) == Some(false) {
        let fact = v.get("fact").cloned().unwrap_or(Value::Null);
        let evidence = v.get("fact_evidence").unwrap_or(&Value::Null);
        let _ = writeln!(
            out,
            "simulate {}  ← no Trellis preview is available for this fact\n",
            str_at(evidence, "frontier")
        );
        let _ = writeln!(out, "  fact:     {}", fact_line(&fact));
        let _ = writeln!(out, "  result:   NOT SIMULATED");
        let _ = writeln!(out, "  reason:   {}", str_at(evidence, "reason"));
        return out;
    }
    let _ = writeln!(
        out,
        "simulate {surface}  ← what committing this fact would do (nothing is applied)\n"
    );
    let fact = v.get("fact").cloned().unwrap_or(Value::Null);
    let _ = writeln!(out, "  fact:     {}", fact_line(&fact));

    let empty = Vec::new();
    let cmds = v
        .get("commands")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if v.get("would_publish").and_then(Value::as_bool) == Some(true) {
        for c in cmds {
            let _ = writeln!(
                out,
                "  result:   WOULD PUBLISH  kind:{}  ({} {})",
                i64_at(c, "kind"),
                str_at(c, "op"),
                str_at(c, "resource"),
            );
        }
    } else if v.get("would_effect").and_then(Value::as_bool) == Some(true) {
        if cmds.is_empty() && i64_at(v, "output_frames") > 0 {
            let _ = writeln!(
                out,
                "  result:   WOULD EMIT     {} output frame(s)",
                i64_at(v, "output_frames")
            );
        } else {
            for c in cmds {
                let _ = writeln!(
                    out,
                    "  result:   WOULD APPLY    ({} {})",
                    str_at(c, "op"),
                    str_at(c, "resource"),
                );
            }
        }
    } else {
        let _ = writeln!(out, "  result:   NO CHANGE (deduped)");
    }
    let changed = strs(v, "changed");
    if !changed.is_empty() {
        let _ = writeln!(out, "  changed:  {}", changed.join(", "));
    }
    out
}

fn fact_line(fact: &Value) -> String {
    let mut parts = Vec::new();
    if let Some(t) = fact.get("title").and_then(Value::as_str) {
        parts.push(format!("title={t:?}"));
    }
    if !parts.is_empty() || fact.get("kind").is_some() {
        return format!(
            "{} → {}",
            fact.get("kind").and_then(Value::as_str).unwrap_or("fact"),
            if parts.is_empty() {
                "(no fields)".to_string()
            } else {
                parts.join(", ")
            }
        );
    }
    serde_json::to_string(fact).unwrap_or_else(|_| "fact".into())
}
