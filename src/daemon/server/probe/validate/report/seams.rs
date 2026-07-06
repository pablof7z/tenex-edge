use super::{int_at, labels_at, str_at};
use serde_json::Value;

pub(in crate::daemon::server::probe::validate) fn seams_status(
    v: &Value,
    surface: Option<&str>,
) -> &'static str {
    if let Some(row) = surface.and_then(|surface| surface_row(v, surface)) {
        if surface_seam_proven(row) {
            "passed"
        } else {
            "not_proven"
        }
    } else if int_at(v, "host_seam_coverage_percent") == 100 {
        "passed"
    } else {
        "not_proven"
    }
}

pub(in crate::daemon::server::probe::validate) fn seams_summary(
    v: &Value,
    surface: Option<&str>,
) -> String {
    if let Some(row) = surface.and_then(|surface| surface_row(v, surface)) {
        return surface_seams_summary(row);
    }

    let risks = bypass_risks(v);
    let unproven = unproven_surfaces(v);
    let suffix = if !risks.is_empty() {
        format!("bypass risks: {}", risks.join("; "))
    } else if !unproven.is_empty() {
        format!("unproven surfaces: {}", unproven.join(", "))
    } else {
        "no declared bypass risks".to_string()
    };
    format!(
        "host seam coverage {}%; {}",
        int_at(v, "host_seam_coverage_percent"),
        suffix
    )
}

fn surface_row<'a>(v: &'a Value, surface: &str) -> Option<&'a Value> {
    v.get("surfaces")
        .and_then(Value::as_array)?
        .iter()
        .find(|row| str_at(row, "surface") == surface)
}

fn surface_seam_proven(row: &Value) -> bool {
    matches!(str_at(row, "mode"), "authoritative" | "projection-owned")
        && labels_at(row, "bypass_risks").is_empty()
}

fn surface_seams_summary(row: &Value) -> String {
    let surface = str_at(row, "surface");
    let mode = str_at(row, "mode");
    let risks = labels_at(row, "bypass_risks");
    if !risks.is_empty() {
        format!(
            "{surface} seam is {mode}; bypass risks: {}",
            risks.join("; ")
        )
    } else if surface_seam_proven(row) {
        format!("{surface} seam is {mode}; no declared bypass risks")
    } else {
        format!("{surface} seam is {mode}; host effects are not fully proven")
    }
}

fn bypass_risks(v: &Value) -> Vec<String> {
    let mut risks = labels_at(v, "uncovered");
    risks.extend(
        v.get("surfaces")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|surface| labels_at(surface, "bypass_risks")),
    );
    risks.sort();
    risks.dedup();
    risks
}

fn unproven_surfaces(v: &Value) -> Vec<String> {
    let mut labels = v
        .get("surfaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|surface| {
            let mode = str_at(surface, "mode");
            if mode == "authoritative" || mode == "projection-owned" {
                None
            } else {
                Some(format!("{} ({mode})", str_at(surface, "surface")))
            }
        })
        .collect::<Vec<_>>();
    labels.sort();
    labels.dedup();
    labels
}
