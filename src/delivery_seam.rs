//! Host seam for mention-delivery policy.

use anyhow::Result;

use crate::reconcile::{DeliveryEffect, DeliveryScanFact};
use crate::state::Store;

pub(crate) fn drive(
    store: &std::sync::Mutex<Store>,
    fact: DeliveryScanFact,
) -> Result<Vec<DeliveryEffect>> {
    let decision = crate::reconcile::delivery::decide(&fact);
    if let Some(decision) = &decision {
        record_receipts(store, decision);
    }
    Ok(crate::reconcile::delivery::effects(decision.as_ref()))
}

fn record_receipts(
    store: &std::sync::Mutex<Store>,
    decision: &crate::reconcile::delivery::DeliveryDecision,
) {
    let created_at = crate::instrument::now_millis();
    let summary = serde_json::json!({
        "pubkey": decision.pubkey,
        "event_ids": decision.event_ids,
        "action": decision.action.as_str(),
        "retry_after_secs": decision.retry_after_secs,
    })
    .to_string();
    let commands = serde_json::json!([{
        "action": decision.action.as_str(),
        "endpoint_id": decision.endpoint_id,
        "retry_after_secs": decision.retry_after_secs,
    }])
    .to_string();
    let guard = store.lock().expect("store mutex poisoned");
    for event_id in &decision.event_ids {
        crate::instrument::record_receipt(
            &guard,
            crate::state::receipts::NewReceipt {
                surface: "delivery".into(),
                transaction_id: created_at,
                revision: 0,
                changed_summary: summary.clone(),
                commands: commands.clone(),
                artifact_ref: Some(event_id.clone()),
                created_at,
            },
        );
    }
}
