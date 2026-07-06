//! `probe validate`: one honest validation envelope over the existing Trellis
//! probes. It gathers oracle/seam/resource checks for every request, then adds
//! target-specific explanation, preview, acid, and replay evidence when the
//! caller supplies a handle, fact, or replay capsule.

use self::label::{cause_label_evidence, malformed_planner_label_evidence};
use self::params::{
    fact_param_for_validation, has_invalid_parameter, has_value, malformed_parameter_evidence,
    simulate_params,
};
use self::report::{
    acid_summary, bool_at, chosen_cause, drift_surfaces, explain_found, explain_summary,
    oracle_summary, push_check, replay_summary, seams_status, seams_summary, simulate_summary,
    str_at, verdict,
};
use self::resource_path::malformed_resource_path_evidence;
use self::state_check::{all_surface_state_checks, target_state_evidence};
use self::target::{
    capsule_target, empty_handle_evidence, explain_handle_parse_error, handle_target,
    malformed_capsule_target_evidence, malformed_probe_handle_evidence, optional_str,
    surface_target, unsupported_target_evidence,
};
use super::{
    acid, artifact, fact, oracle, replay, seams, simulate, state, stats, why, DaemonState,
};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

mod alias;
mod awareness;
mod channel;
mod commit;
mod coverage;
mod cursor;
mod durable_outbox;
mod envelope;
mod event;
mod hook_context;
mod identity;
mod inbox;
mod joined;
mod label;
mod llm;
mod lookup;
mod membership;
mod message;
mod outbox;
mod params;
mod project_root;
mod quarantine;
mod readiness_attempt;
mod receipt;
mod recipient;
mod report;
mod resource_path;
mod scope;
mod session;
mod session_consistency;
mod session_start;
mod session_watch;
mod state_check;
mod status;
mod subscription;
mod table_samples;
mod target;
mod target_checks;
mod turn;
mod txn;

pub(super) fn validate_value(state: &Arc<DaemonState>, params: &Value) -> Result<Value> {
    let parameter_evidence = malformed_parameter_evidence(params);
    let raw_target = optional_str(params, "target");
    let target = raw_target.filter(|target| *target != "all");
    let explicit_surface = target.and_then(surface_target);
    let malformed_target_evidence = empty_handle_evidence(target)
        .or_else(|| malformed_capsule_target_evidence(target))
        .or_else(|| malformed_probe_handle_evidence(target))
        .or_else(|| malformed_resource_path_evidence(target))
        .or_else(|| explain_handle_parse_error(target))
        .or_else(|| malformed_planner_label_evidence(target));
    let has_malformed_target = malformed_target_evidence.is_some();
    let has_invalid_capsule_parameter = has_invalid_parameter(&parameter_evidence, "capsule");
    let cause_label_evidence = if has_malformed_target {
        None
    } else {
        cause_label_evidence(target)
    };
    let target_checks =
        target_checks::TargetChecks::collect(state, params, target, has_malformed_target);
    let handle = if cause_label_evidence.is_none() && malformed_target_evidence.is_none() {
        target.and_then(handle_target)
    } else {
        None
    };
    let capsule = if has_malformed_target || has_invalid_capsule_parameter {
        None
    } else {
        capsule_target(params, target)
    };
    let explain_handle = if malformed_target_evidence.is_none() {
        target.and_then(|target| crate::explain::parse_handle(target).ok())
    } else {
        None
    };
    let target_evidence = malformed_target_evidence.or_else(|| {
        unsupported_target_evidence(
            target,
            explicit_surface,
            handle,
            capsule,
            explain_handle.is_some(),
            cause_label_evidence.is_some(),
            target_checks.supported(),
        )
    });
    let (fact, invalid_fact_evidence) = fact_param_for_validation(params);
    let fact_surface = fact.as_ref().and_then(artifact::infer_surface);
    let fact_evidence = fact
        .as_ref()
        .map(|fact| fact::fact_evidence(fact, fact_surface))
        .or(invalid_fact_evidence);
    let capsule_surface = capsule.and_then(|capsule| scope::stored_capsule_surface(state, capsule));

    let why = match handle {
        Some(handle) => match why::why_value(state, &json!({ "verb": "why", "handle": handle })) {
            Ok(v) => Some(v),
            Err(e) => Some(json!({
                "verb": "why",
                "handle": handle,
                "found": false,
                "error": e.to_string(),
                "note": e.to_string(),
            })),
        },
        None => None,
    };

    let mut explain_error = None;
    let explanation = match &explain_handle {
        Some(handle) => match state.with_store(|s| crate::explain::explain(s, handle)) {
            Ok(v) => Some(v),
            Err(e) => {
                explain_error = Some(e.to_string());
                None
            }
        },
        None => None,
    };

    let surface = scope::selected_surface(
        target,
        explain_handle.as_ref(),
        &target_checks,
        explanation.as_ref(),
        cause_label_evidence.as_ref(),
        fact_surface,
        capsule_surface,
    );
    let global_validation = raw_target.filter(|target| *target != "all").is_none()
        && parameter_evidence.is_empty()
        && target_evidence.is_none()
        && fact_evidence.is_none()
        && capsule.is_none()
        && surface.is_none();

    let mut checks = Vec::new();
    let mut limitations = Vec::new();

    if !parameter_evidence.is_empty() {
        let names = parameter_evidence
            .iter()
            .filter_map(|v| v.get("parameter").and_then(Value::as_str))
            .collect::<Vec<_>>();
        push_check(
            &mut checks,
            "input",
            "failed",
            format!("invalid validate parameter(s): {}", names.join(", ")),
        );
        limitations.extend(
            parameter_evidence
                .iter()
                .filter_map(|v| v.get("reason").and_then(Value::as_str))
                .map(str::to_string),
        );
    }

    let oracle = oracle::oracle_value(state);
    push_check(
        &mut checks,
        "oracle",
        if bool_at(&oracle, "ok") {
            "passed"
        } else {
            "failed"
        },
        oracle_summary(&oracle),
    );

    let seams = seams::seams_value();
    let global_seams_checked = global_validation || target_checks.global_seams_checked();
    if surface.is_some() || global_seams_checked {
        let seam_status = seams_status(&seams, surface.as_deref());
        push_check(
            &mut checks,
            "seams",
            seam_status,
            seams_summary(&seams, surface.as_deref()),
        );
        if seam_status == "not_proven" {
            if !bool_at(&oracle, "surface_correctness_proven") {
                limitations.push(
                    "oracle proves graph bookkeeping, not host-effect correctness".to_string(),
                );
            }
            limitations.push("some host-effect paths are not covered by Trellis seams".to_string());
        }
    }

    if let Some(v) = &target_evidence {
        let status = if v.get("valid").and_then(Value::as_bool) == Some(false) {
            "failed"
        } else {
            "not_proven"
        };
        push_check(
            &mut checks,
            "target",
            status,
            str_at(v, "summary").to_string(),
        );
        limitations.push(str_at(v, "reason").to_string());
    }

    if let Some(v) = &cause_label_evidence {
        push_check(
            &mut checks,
            "cause_label",
            "passed",
            str_at(v, "summary").to_string(),
        );
    }
    target_checks.push_checks(&mut checks, &mut limitations);

    if let Some(v) = &why {
        durable_outbox::push_why_check(&mut checks, &mut limitations, v, &target_checks);
    }

    if let Some(v) = &explanation {
        let explain_status = if explain_found(v) {
            "passed"
        } else if target_checks.event_checked() || target_checks.txn_checked() {
            "not_proven"
        } else {
            "failed"
        };
        push_check(&mut checks, "explain", explain_status, explain_summary(v));
        if explain_status == "not_proven" {
            limitations
                .push("no Trellis receipt/LLM explanation is recorded for this target".to_string());
        }
    }
    if let Some(error) = &explain_error {
        push_check(&mut checks, "explain", "failed", error.clone());
    }

    if let Some(v) = &fact_evidence {
        if bool_at(v, "supported") {
            push_check(
                &mut checks,
                "fact",
                "passed",
                str_at(v, "summary").to_string(),
            );
        } else if v.get("valid").and_then(Value::as_bool) == Some(false) {
            push_check(
                &mut checks,
                "fact",
                "failed",
                str_at(v, "summary").to_string(),
            );
            limitations.push(str_at(v, "reason").to_string());
        } else {
            push_check(
                &mut checks,
                "fact",
                "not_proven",
                str_at(v, "summary").to_string(),
            );
            limitations.push(str_at(v, "reason").to_string());
        }
    }

    let mut stats_error = None;
    let stats = if surface.is_some() || global_seams_checked {
        let since = params.get("since").and_then(Value::as_i64).unwrap_or(0);
        match state.with_store(|s| stats::stats_value(s, surface.as_deref(), since)) {
            Ok(v) => {
                let drift = drift_surfaces(&v);
                push_check(
                    &mut checks,
                    "resource_accounting",
                    if drift.is_empty() { "passed" } else { "failed" },
                    if drift.is_empty() {
                        "no resource drift in commit ledger".to_string()
                    } else {
                        format!("resource drift: {}", drift.join(", "))
                    },
                );
                Some(v)
            }
            Err(e) => {
                let error = e.to_string();
                push_check(&mut checks, "resource_accounting", "failed", error.clone());
                stats_error = Some(error);
                None
            }
        }
    } else {
        None
    };

    let mut state_error = None;
    let surface_state = if has_malformed_target {
        None
    } else {
        match surface.as_deref() {
            Some(surface) => {
                match state::state_value(state, &json!({ "verb": "state", "surface": surface })) {
                    Ok(v) => {
                        let (status, summary) = state_check::state_check_summary(&v, None, None);
                        Some(state_check::annotated_surface_state(v, status, &summary))
                    }
                    Err(e) => {
                        state_error = Some(e.to_string());
                        None
                    }
                }
            }
            None => None,
        }
    };
    if let Some(v) = &surface_state {
        durable_outbox::push_state_check(&mut checks, v, handle, why.as_ref(), &target_checks);
    }
    if let Some(error) = &state_error {
        push_check(&mut checks, "state", "failed", error.clone());
    }
    let state_evidence = if target_checks.supported() {
        None
    } else {
        surface_state
            .as_ref()
            .and_then(|v| target_state_evidence(v, handle, why.as_ref()))
    };
    let (state_checks, surface_states) = if global_validation {
        all_surface_state_checks(state)
    } else {
        (Vec::new(), Vec::new())
    };
    checks.extend(state_checks);
    let session_consistency = if global_validation {
        let evidence = session_consistency::session_consistency_evidence(state);
        session_consistency::push_session_consistency_check(
            &mut checks,
            &mut limitations,
            &evidence,
        );
        Some(evidence)
    } else {
        None
    };

    let mut simulate_error = None;
    let simulation = if has_value(params, "fact") && fact_surface.is_some() {
        let sim_params = simulate_params(params, surface.as_deref());
        match simulate::simulate_value(state, &sim_params) {
            Ok(v) => {
                push_check(&mut checks, "simulate", "passed", simulate_summary(&v));
                Some(v)
            }
            Err(e) => {
                let error = e.to_string();
                push_check(&mut checks, "simulate", "not_proven", error.clone());
                limitations.push("simulation could not be proven for this fact".to_string());
                simulate_error = Some(error);
                None
            }
        }
    } else {
        None
    };

    let mut acid_error = None;
    let acid = match (handle, &simulation) {
        (Some(handle), Some(simulation)) => {
            let cause = chosen_cause(optional_str(params, "cause"), simulation, why.as_ref());
            let mut acid_params = json!({
                "verb": "acid",
                "handle": handle,
                "fact": params.get("fact").cloned().unwrap_or(Value::Null),
            });
            if let Some(cause) = &cause {
                acid_params["cause"] = Value::String(cause.clone());
            }
            match acid::acid_value(state, &acid_params) {
                Ok(v) => {
                    push_check(
                        &mut checks,
                        "acid",
                        if bool_at(&v, "ok") {
                            "passed"
                        } else {
                            "failed"
                        },
                        acid_summary(&v),
                    );
                    Some(v)
                }
                Err(e) => {
                    let error = e.to_string();
                    push_check(&mut checks, "acid", "not_proven", error.clone());
                    limitations
                        .push("acid necessity could not be proven for this fact/handle".into());
                    acid_error = Some(error);
                    None
                }
            }
        }
        _ => None,
    };

    let mut replay_error = None;
    let replay = match capsule {
        Some(capsule) => {
            let params = json!({ "verb": "replay", "capsule": capsule, "assert": true });
            match replay::replay_value(state, &params) {
                Ok(v) => {
                    push_check(
                        &mut checks,
                        "replay",
                        if bool_at(&v, "ok") && bool_at(&v, "asserted") {
                            "passed"
                        } else {
                            "failed"
                        },
                        replay_summary(&v),
                    );
                    Some(v)
                }
                Err(e) => {
                    let error = e.to_string();
                    push_check(&mut checks, "replay", "failed", error.clone());
                    replay_error = Some(error);
                    None
                }
            }
        }
        None => None,
    };

    let verdict = verdict(&checks, &limitations);
    Ok(envelope::build(
        raw_target,
        surface,
        handle,
        target.filter(|_| explain_handle.is_some()),
        capsule,
        verdict,
        checks,
        limitations,
        oracle,
        seams,
        stats,
        stats_error,
        surface_state,
        state_evidence,
        session_consistency,
        surface_states,
        state_error,
        why,
        explanation,
        explain_error,
        parameter_evidence,
        target_evidence,
        cause_label_evidence,
        target_checks,
        fact_evidence,
        simulation,
        simulate_error,
        acid,
        acid_error,
        replay,
        replay_error,
    ))
}
