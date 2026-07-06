//! Validator coverage inventory for durable ledgers and Trellis surfaces.

use super::report::str_at;
use super::DaemonState;
use crate::reconcile::frontier;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

mod catalog;
use catalog::{surface_targets, table_coverage, TableCoverage, DURABLE_TABLES};

pub(super) enum CoverageTarget {
    Inventory,
    Table(String),
    Lookup(String),
}

pub(super) fn coverage_target(target: &str) -> Option<CoverageTarget> {
    if matches!(
        target,
        "coverage"
            | "coverage:all"
            | "validation_coverage"
            | "validation-coverage"
            | "inventory"
            | "table_inventory"
            | "table-inventory"
    ) {
        return Some(CoverageTarget::Inventory);
    }
    target
        .strip_prefix("table:")
        .or_else(|| target.strip_prefix("table/"))
        .or_else(|| target.strip_prefix("ledger:"))
        .or_else(|| target.strip_prefix("ledger/"))
        .and_then(|rest| rest.split('/').next())
        .filter(|table| !table.trim().is_empty())
        .map(|table| CoverageTarget::Table(table.to_string()))
        .or_else(|| super::lookup::explicit_lookup_target(target).map(CoverageTarget::Lookup))
        .or_else(|| super::lookup::bare_lookup_target(target).map(CoverageTarget::Lookup))
}

pub(super) fn coverage_evidence(
    state: &Arc<DaemonState>,
    target: &str,
    parsed: &CoverageTarget,
) -> Value {
    match parsed {
        CoverageTarget::Inventory => inventory_evidence(state, target),
        CoverageTarget::Table(table) => table_evidence(state, target, table),
        CoverageTarget::Lookup(needle) => super::lookup::lookup_evidence(state, target, needle),
    }
}

fn inventory_evidence(state: &Arc<DaemonState>, target: &str) -> Value {
    let live_tables = match state.with_store(|store| store.application_table_names()) {
        Ok(tables) => tables,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "validation_coverage",
                "supported": true,
                "ok": false,
                "coverage_ok": false,
                "error": e.to_string(),
                "summary": "validation coverage could not read the durable table inventory",
                "reason": e.to_string(),
            });
        }
    };
    let live = live_tables.iter().cloned().collect::<BTreeSet<_>>();
    let catalog = DURABLE_TABLES
        .iter()
        .map(|row| row.table.to_string())
        .collect::<BTreeSet<_>>();
    let uncovered = live.difference(&catalog).cloned().collect::<Vec<_>>();
    let not_present = catalog.difference(&live).cloned().collect::<Vec<_>>();
    let covered_table_count = live.intersection(&catalog).count();
    let direct_table_count = DURABLE_TABLES
        .iter()
        .filter(|row| row.mode == "direct" && live.contains(row.table))
        .count();
    let coverage_ok = uncovered.is_empty();

    json!({
        "target": target,
        "kind": "validation_coverage",
        "supported": true,
        "ok": coverage_ok,
        "coverage_ok": coverage_ok,
        "table_count": live.len(),
        "catalog_table_count": DURABLE_TABLES.len(),
        "covered_table_count": covered_table_count,
        "direct_table_count": direct_table_count,
        "aggregate_table_count": covered_table_count.saturating_sub(direct_table_count),
        "uncovered_tables": uncovered,
        "known_tables_not_present": not_present,
        "durable_tables": DURABLE_TABLES.iter().map(|row| json!({
            "table": row.table,
            "mode": row.mode,
            "targets": row.targets,
            "proves": row.proves,
            "present": live.contains(row.table),
        })).collect::<Vec<_>>(),
        "surfaces": frontier::registrations().iter().map(|row| json!({
            "surface": row.name,
            "mode": row.mode.as_str(),
            "targets": surface_targets(row.name),
            "host_effects": row.host_effects,
            "bypass_risks": row.bypass_risks,
        })).collect::<Vec<_>>(),
        "summary": inventory_summary(live.len(), covered_table_count, coverage_ok),
        "reason": inventory_reason(coverage_ok),
    })
}

fn table_evidence(state: &Arc<DaemonState>, target: &str, table: &str) -> Value {
    let result = state.with_store(|store| {
        Ok::<_, anyhow::Error>((
            store.application_table_names()?,
            store.application_table_profile(table)?,
            super::table_samples::sample_targets(store, table, 5)?,
        ))
    });
    let (live_tables, profile, sample_targets) = match result {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "target": target,
                "kind": "validation_table",
                "table": table,
                "supported": true,
                "found": false,
                "present": false,
                "covered": false,
                "ok": false,
                "error": e.to_string(),
                "summary": format!("table `{table}` evidence could not read durable state"),
                "reason": e.to_string(),
            });
        }
    };
    let coverage = table_coverage(table);
    let present = profile.is_some();
    let covered = coverage.is_some();
    let (row_count, columns) = profile.unwrap_or_else(|| (0, Vec::new()));

    json!({
        "target": target,
        "kind": "validation_table",
        "table": table,
        "supported": true,
        "found": present,
        "present": present,
        "covered": covered,
        "ok": present && covered,
        "row_count": row_count,
        "column_count": columns.len(),
        "columns": columns,
        "mode": coverage.map(|row| row.mode).unwrap_or(""),
        "targets": coverage.map(|row| row.targets).unwrap_or(""),
        "proves": coverage.map(|row| row.proves).unwrap_or(""),
        "live_table_count": live_tables.len(),
        "known_tables": live_tables,
        "sample_target_count": sample_targets.len(),
        "sample_targets": sample_targets,
        "summary": table_summary(table, present, row_count, coverage),
        "reason": table_reason(table, present, coverage),
    })
}

pub(super) fn push_coverage_check(
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
    evidence: &Value,
) {
    if str_at(evidence, "kind") == "validation_table" {
        push_table_check(checks, limitations, evidence);
        return;
    }
    if str_at(evidence, "kind") == "validation_lookup" {
        super::lookup::push_lookup_check(checks, limitations, evidence);
        return;
    }

    let status = if evidence.get("coverage_ok").and_then(Value::as_bool) == Some(true) {
        "passed"
    } else {
        "failed"
    };
    checks.push(json!({
        "name": "validation_coverage",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn push_table_check(checks: &mut Vec<Value>, limitations: &mut Vec<String>, evidence: &Value) {
    let present = evidence.get("present").and_then(Value::as_bool) == Some(true);
    let covered = evidence.get("covered").and_then(Value::as_bool) == Some(true);
    let status = if !str_at(evidence, "error").is_empty() || (present && !covered) {
        "failed"
    } else if present && covered {
        "passed"
    } else {
        "not_proven"
    };
    checks.push(json!({
        "name": "table_coverage",
        "status": status,
        "summary": str_at(evidence, "summary"),
    }));
    if status != "passed" && !str_at(evidence, "reason").is_empty() {
        limitations.push(str_at(evidence, "reason").to_string());
    }
}

fn inventory_summary(table_count: usize, covered_table_count: usize, coverage_ok: bool) -> String {
    if coverage_ok {
        format!("validator maps {covered_table_count}/{table_count} live durable table(s)")
    } else {
        format!(
            "validator maps {covered_table_count}/{table_count} live durable table(s); uncovered durable tables remain"
        )
    }
}

fn inventory_reason(coverage_ok: bool) -> String {
    if coverage_ok {
        return "every live durable application table has a declared validation target family"
            .into();
    }
    "one or more live durable tables has no declared validation target family".into()
}

fn table_summary(
    table: &str,
    present: bool,
    row_count: i64,
    coverage: Option<&TableCoverage>,
) -> String {
    match (present, coverage) {
        (true, Some(row)) => format!(
            "table `{table}` has {row_count} row(s) and maps to `{}`",
            row.targets
        ),
        (true, None) => {
            format!("table `{table}` has {row_count} row(s) but no validation target family")
        }
        (false, _) => format!("table `{table}` is not a live durable application table"),
    }
}

fn table_reason(table: &str, present: bool, coverage: Option<&TableCoverage>) -> String {
    match (present, coverage) {
        (true, Some(row)) => format!(
            "use `{}` to validate rows from `{table}`; this table proves {}",
            row.targets, row.proves
        ),
        (true, None) => "live durable table is absent from the validation coverage catalog".into(),
        (false, _) => "no sqlite application table matched this name".into(),
    }
}
