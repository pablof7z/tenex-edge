//! Human renders for the differentiated probe verbs. Each is scrupulously honest
//! about what the daemon proved (frontier §4.6): the oracle render names what is
//! NOT proven and lists the uncovered surfaces; simulate says plainly that
//! nothing was applied; why prints the latest-per-key footer.

use serde_json::Value;
use std::fmt::Write as _;

#[cfg(test)]
pub(super) use super::state_render::render_state;

#[cfg(test)]
mod tests;

mod simulate;
pub(super) use simulate::render_simulate;

fn str_at<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn i64_at(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}
fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// `probe oracle` — per-surface green/red + the honest correctness caveat.
pub(super) fn render_oracle(v: &Value) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "oracle  (live)");
    let empty = Vec::new();
    for s in v
        .get("surfaces")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        let name = str_at(s, "surface");
        if s.get("live_graph").and_then(Value::as_bool) == Some(false) {
            let _ = writeln!(out, "  {name:<15} —        not a live graph (advisory)");
            continue;
        }
        let status = str_at(s, "status");
        let rev = i64_at(s, "revision");
        let nodes = i64_at(s, "nodes");
        let _ = writeln!(out, "  {name:<15} {status:<8} (rev {rev}, {nodes} nodes)");
        if status == "red" {
            let _ = writeln!(out, "      ↳ {}", str_at(s, "error"));
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "oracle: {} / surface-correctness: {} / host-seam-coverage: {}% / uncovered: {}",
        str_at(v, "oracle"),
        str_at(v, "surface_correctness"),
        i64_at(v, "host_seam_coverage_percent"),
        strs(v, "uncovered").join(", "),
    );
    let _ = writeln!(
        out,
        "surface-correctness detail: oracle checks graph bookkeeping, not host effects"
    );
    let _ = writeln!(out, "covered: {}", strs(v, "covered").join(", "));
    out
}

/// `probe seams` — static authority-frontier registrations plus coverage.
pub(super) fn render_seams(v: &Value) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "seams  (host-seam-coverage: {}%)",
        i64_at(v, "host_seam_coverage_percent")
    );
    let _ = writeln!(out, "{:<16} {:<17} bypass risks", "surface", "mode");
    let empty = Vec::new();
    for row in v
        .get("surfaces")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        let risks = strs(row, "bypass_risks").join(", ");
        let _ = writeln!(
            out,
            "{:<16} {:<17} {}",
            str_at(row, "surface"),
            str_at(row, "mode"),
            if risks.is_empty() {
                "-"
            } else {
                risks.as_str()
            },
        );
    }
    out
}

/// `probe replay <capsule>` — stored input capsule metadata + replay result.
pub(super) fn render_replay(v: &Value) -> String {
    let mut out = String::new();
    let capsule = v.get("capsule").unwrap_or(&Value::Null);
    let id = i64_at(capsule, "id");
    let surface = str_at(capsule, "surface");
    let trigger = str_at(capsule, "trigger_kind");
    let trigger_ref = str_at(capsule, "trigger_ref");
    let _ = writeln!(
        out,
        "replay capsule {id}  ({surface}/{trigger} {trigger_ref})"
    );
    let _ = writeln!(
        out,
        "  stored:   {} bytes, trace format v{}",
        i64_at(capsule, "script_bytes"),
        i64_at(capsule, "format_version"),
    );
    if v.get("asserted").and_then(Value::as_bool) == Some(true) {
        let _ = writeln!(
            out,
            "  assert:   ok  ({} steps, {} commands, {} frames)",
            i64_at(v, "steps"),
            i64_at(v, "resource_commands"),
            i64_at(v, "output_frames"),
        );
    } else {
        let _ = writeln!(out, "  assert:   not run");
    }
    if let Some(path) = v.get("trace_path").and_then(Value::as_str) {
        let _ = writeln!(out, "  trace:    {path}");
    }
    out
}

/// `probe diff` — before/after artifact hashes plus changed fields.
pub(super) fn render_diff(v: &Value) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "diff {}  ({})",
        str_at(v, "surface"),
        str_at(v, "mode")
    );
    let verdict = if v.get("artifact_changed").and_then(Value::as_bool) == Some(true) {
        "CHANGED"
    } else {
        "UNCHANGED"
    };
    let _ = writeln!(out, "  artifact: {verdict}");
    let _ = writeln!(out, "  before:   {}", str_at(v, "before_hash"));
    let _ = writeln!(out, "  after:    {}", str_at(v, "after_hash"));
    let empty = Vec::new();
    for row in v
        .get("field_diff")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        let _ = writeln!(out, "  changed:  {}", str_at(row, "field"));
    }
    out
}

/// `probe acid` — necessity and unrelated-input stability verdicts.
pub(super) fn render_acid(v: &Value) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "acid {}  ({})",
        str_at(v, "handle"),
        str_at(v, "surface")
    );
    let _ = writeln!(out, "  cause:    {}", str_at(v, "cause"));
    let _ = writeln!(
        out,
        "  verdict:  necessary={}  unrelated_stable={}  ok={}",
        v.get("necessary").and_then(Value::as_bool).unwrap_or(false),
        v.get("unrelated_stable")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        v.get("ok").and_then(Value::as_bool).unwrap_or(false),
    );
    let _ = writeln!(out, "  original: {}", str_at(v, "original_hash"));
    let _ = writeln!(out, "  removed:  {}", str_at(v, "removed_hash"));
    let _ = writeln!(out, "  unrelated: {}", str_at(v, "unrelated_hash"));
    out
}

/// `probe why <handle>` — live causality + the latest-per-key footer.
pub(super) fn render_why(v: &Value) -> String {
    let mut out = String::new();
    let handle = str_at(v, "handle");
    let _ = writeln!(out, "why {handle}");

    if v.get("found").and_then(Value::as_bool) != Some(true) {
        let note = v
            .get("note")
            .and_then(Value::as_str)
            .unwrap_or("no live audit for this handle");
        let _ = writeln!(out, "  {note}");
        return out;
    }

    let _ = writeln!(out, "  resource:  {}", str_at(v, "resource_key"));
    if str_at(v, "kind") == "subscription" {
        let owners = strs(v, "owners").join(", ");
        let _ = writeln!(
            out,
            "  owners:    {owners}   (refcount {})",
            i64_at(v, "refcount")
        );
    }
    let _ = writeln!(
        out,
        "  last:      {}  ← {}",
        str_at(v, "last_kind"),
        str_at(v, "cause")
    );
    let causes = strs(v, "input_causes");
    if !causes.is_empty() {
        let _ = writeln!(out, "  caused by: {}", causes.join(", "));
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "(latest change per key; for history use probe stats / the commits ledger)"
    );
    out
}
