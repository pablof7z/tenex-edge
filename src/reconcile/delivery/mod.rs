//! Pure mention-delivery policy.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryScanFact {
    pub pubkey: String,
    pub pending_event_ids: Vec<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_live: bool,
    pub last_injected_at: Option<u64>,
    pub debounce_secs: u64,
    pub force: bool,
    pub at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeliveryAction {
    Inject,
    DeferDebounced,
    DeferNoEndpoint,
    ClearDeadEndpoint,
}

impl DeliveryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inject => "inject",
            Self::DeferDebounced => "defer_debounced",
            Self::DeferNoEndpoint => "defer_no_endpoint",
            Self::ClearDeadEndpoint => "clear_dead_endpoint",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryDecision {
    pub pubkey: String,
    pub action: DeliveryAction,
    pub event_ids: Vec<String>,
    pub endpoint_id: Option<String>,
    pub retry_after_secs: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeliveryEffect {
    Inject {
        pubkey: String,
        endpoint_id: String,
        event_ids: Vec<String>,
    },
    RetryAfter {
        pubkey: String,
        delay_secs: u64,
    },
    ClearDeadEndpoint {
        pubkey: String,
    },
}

pub fn decide(fact: &DeliveryScanFact) -> Option<DeliveryDecision> {
    if fact.pending_event_ids.is_empty() {
        return None;
    }
    let Some(endpoint_id) = fact.endpoint_id.clone() else {
        return Some(decision(fact, DeliveryAction::DeferNoEndpoint, None, None));
    };
    if !fact.endpoint_live {
        return Some(decision(
            fact,
            DeliveryAction::ClearDeadEndpoint,
            Some(endpoint_id),
            None,
        ));
    }
    let last = fact.last_injected_at.unwrap_or(0);
    let elapsed = fact.at.saturating_sub(last);
    if !fact.force && last > 0 && elapsed < fact.debounce_secs {
        return Some(decision(
            fact,
            DeliveryAction::DeferDebounced,
            Some(endpoint_id),
            Some(fact.debounce_secs.saturating_sub(elapsed).max(1)),
        ));
    }
    Some(decision(
        fact,
        DeliveryAction::Inject,
        Some(endpoint_id),
        None,
    ))
}

pub fn effects(decision: Option<&DeliveryDecision>) -> Vec<DeliveryEffect> {
    let Some(decision) = decision else {
        return Vec::new();
    };
    match decision.action {
        DeliveryAction::Inject => decision
            .endpoint_id
            .clone()
            .map(|endpoint_id| DeliveryEffect::Inject {
                pubkey: decision.pubkey.clone(),
                endpoint_id,
                event_ids: decision.event_ids.clone(),
            })
            .into_iter()
            .collect(),
        DeliveryAction::DeferDebounced => decision
            .retry_after_secs
            .map(|delay_secs| DeliveryEffect::RetryAfter {
                pubkey: decision.pubkey.clone(),
                delay_secs,
            })
            .into_iter()
            .collect(),
        DeliveryAction::ClearDeadEndpoint => vec![DeliveryEffect::ClearDeadEndpoint {
            pubkey: decision.pubkey.clone(),
        }],
        DeliveryAction::DeferNoEndpoint => Vec::new(),
    }
}

fn decision(
    fact: &DeliveryScanFact,
    action: DeliveryAction,
    endpoint_id: Option<String>,
    retry_after_secs: Option<u64>,
) -> DeliveryDecision {
    DeliveryDecision {
        pubkey: fact.pubkey.clone(),
        action,
        event_ids: fact.pending_event_ids.clone(),
        endpoint_id,
        retry_after_secs,
    }
}
