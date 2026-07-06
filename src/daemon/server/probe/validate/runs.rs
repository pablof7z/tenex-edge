use super::params::{has_value, simulate_params};
use super::report::{
    acid_summary, bool_at, chosen_cause, push_check, replay_summary, simulate_summary,
};
use super::target::optional_str;
use super::DaemonState;
use serde_json::{json, Value};
use std::sync::Arc;

pub(super) struct ActiveRuns {
    pub(super) simulation: Option<Value>,
    pub(super) simulate_error: Option<String>,
    pub(super) acid: Option<Value>,
    pub(super) acid_error: Option<String>,
    pub(super) replay: Option<Value>,
    pub(super) replay_error: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect(
    state: &Arc<DaemonState>,
    params: &Value,
    surface: Option<&str>,
    fact_surface: Option<&str>,
    handle: Option<&str>,
    why: Option<&Value>,
    capsule: Option<&str>,
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
) -> ActiveRuns {
    let (simulation, simulate_error) =
        simulation(state, params, surface, fact_surface, checks, limitations);
    let (acid, acid_error) = acid(state, params, handle, why, &simulation, checks, limitations);
    let (replay, replay_error) = replay(state, capsule, checks);
    ActiveRuns {
        simulation,
        simulate_error,
        acid,
        acid_error,
        replay,
        replay_error,
    }
}

fn simulation(
    state: &Arc<DaemonState>,
    params: &Value,
    surface: Option<&str>,
    fact_surface: Option<&str>,
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
) -> (Option<Value>, Option<String>) {
    if !(has_value(params, "fact") && fact_surface.is_some()) {
        return (None, None);
    }
    let sim_params = simulate_params(params, surface);
    match super::super::simulate::simulate_value(state, &sim_params) {
        Ok(v) => {
            push_check(checks, "simulate", "passed", simulate_summary(&v));
            (Some(v), None)
        }
        Err(e) => {
            let error = e.to_string();
            push_check(checks, "simulate", "not_proven", error.clone());
            limitations.push("simulation could not be proven for this fact".to_string());
            (None, Some(error))
        }
    }
}

fn acid(
    state: &Arc<DaemonState>,
    params: &Value,
    handle: Option<&str>,
    why: Option<&Value>,
    simulation: &Option<Value>,
    checks: &mut Vec<Value>,
    limitations: &mut Vec<String>,
) -> (Option<Value>, Option<String>) {
    let (Some(handle), Some(simulation)) = (handle, simulation.as_ref()) else {
        return (None, None);
    };
    let cause = chosen_cause(optional_str(params, "cause"), simulation, why);
    let mut acid_params = json!({
        "verb": "acid",
        "handle": handle,
        "fact": params.get("fact").cloned().unwrap_or(Value::Null),
    });
    if let Some(cause) = &cause {
        acid_params["cause"] = Value::String(cause.clone());
    }
    match super::super::acid::acid_value(state, &acid_params) {
        Ok(v) => {
            push_check(
                checks,
                "acid",
                if bool_at(&v, "ok") {
                    "passed"
                } else {
                    "failed"
                },
                acid_summary(&v),
            );
            (Some(v), None)
        }
        Err(e) => {
            let error = e.to_string();
            push_check(checks, "acid", "not_proven", error.clone());
            limitations.push("acid necessity could not be proven for this fact/handle".into());
            (None, Some(error))
        }
    }
}

fn replay(
    state: &Arc<DaemonState>,
    capsule: Option<&str>,
    checks: &mut Vec<Value>,
) -> (Option<Value>, Option<String>) {
    let Some(capsule) = capsule else {
        return (None, None);
    };
    let params = json!({ "verb": "replay", "capsule": capsule, "assert": true });
    match super::super::replay::replay_value(state, &params) {
        Ok(v) => {
            push_check(
                checks,
                "replay",
                if bool_at(&v, "ok") && bool_at(&v, "asserted") {
                    "passed"
                } else {
                    "failed"
                },
                replay_summary(&v),
            );
            (Some(v), None)
        }
        Err(e) => {
            let error = e.to_string();
            push_check(checks, "replay", "failed", error.clone());
            (None, Some(error))
        }
    }
}
