use super::label::{cause_label_evidence, malformed_planner_label_evidence};
use super::params::{
    fact_param_for_validation, has_invalid_parameter, malformed_parameter_evidence,
};
use super::resource_path::malformed_resource_path_evidence;
use super::target::{
    capsule_target, empty_handle_evidence, explain_handle_parse_error, handle_target,
    malformed_capsule_target_evidence, malformed_probe_handle_evidence, optional_str,
    surface_target, unsupported_target_evidence,
};
use super::target_checks::TargetChecks;
use super::{scope, DaemonState};
use serde_json::Value;
use std::sync::Arc;

pub(super) struct ValidationInput<'a> {
    pub(super) parameter_evidence: Vec<Value>,
    pub(super) raw_target: Option<&'a str>,
    pub(super) target: Option<&'a str>,
    pub(super) has_malformed_target: bool,
    pub(super) cause_label_evidence: Option<Value>,
    pub(super) target_checks: TargetChecks,
    pub(super) handle: Option<&'a str>,
    pub(super) capsule: Option<&'a str>,
    pub(super) explain_handle: Option<crate::explain::Handle>,
    pub(super) target_evidence: Option<Value>,
    pub(super) fact_surface: Option<&'static str>,
    pub(super) fact_evidence: Option<Value>,
    pub(super) capsule_surface: Option<String>,
}

impl<'a> ValidationInput<'a> {
    pub(super) fn collect(state: &Arc<DaemonState>, params: &'a Value) -> Self {
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
        let target_checks = TargetChecks::collect(state, params, target, has_malformed_target);
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
        let fact_surface = fact
            .as_ref()
            .and_then(super::super::artifact::infer_surface);
        let fact_evidence = fact
            .as_ref()
            .map(|fact| super::super::fact::fact_evidence(fact, fact_surface))
            .or(invalid_fact_evidence);
        let capsule_surface =
            capsule.and_then(|capsule| scope::stored_capsule_surface(state, capsule));

        Self {
            parameter_evidence,
            raw_target,
            target,
            has_malformed_target,
            cause_label_evidence,
            target_checks,
            handle,
            capsule,
            explain_handle,
            target_evidence,
            fact_surface,
            fact_evidence,
            capsule_surface,
        }
    }
}
