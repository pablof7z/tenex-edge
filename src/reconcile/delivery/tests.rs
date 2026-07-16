use super::*;

fn fact() -> DeliveryScanFact {
    DeliveryScanFact {
        pubkey: "pk".into(),
        pending_event_ids: vec!["event".into()],
        endpoint_id: Some("pty".into()),
        endpoint_live: true,
        last_injected_at: None,
        debounce_secs: 20,
        force: false,
        at: 100,
    }
}

#[test]
fn injects_when_endpoint_is_live_and_not_debounced() {
    let decision = decide(&fact()).unwrap();
    assert_eq!(decision.action, DeliveryAction::Inject);
    assert!(matches!(
        effects(Some(&decision))[0],
        DeliveryEffect::Inject { .. }
    ));
}

#[test]
fn defers_inside_debounce_window() {
    let mut input = fact();
    input.last_injected_at = Some(90);
    let decision = decide(&input).unwrap();
    assert_eq!(decision.action, DeliveryAction::DeferDebounced);
    assert_eq!(decision.retry_after_secs, Some(10));
}

#[test]
fn clears_dead_endpoint_and_ignores_empty_scan() {
    let mut input = fact();
    input.endpoint_live = false;
    assert!(matches!(
        effects(decide(&input).as_ref())[0],
        DeliveryEffect::ClearDeadEndpoint { .. }
    ));
    input.pending_event_ids.clear();
    assert!(decide(&input).is_none());
}
