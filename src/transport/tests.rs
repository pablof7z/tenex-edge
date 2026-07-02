use super::*;

// ── stable subscription id ──────────────────────────────────────────────────

#[test]
fn stable_subscription_id_is_deterministic_and_distinct() {
    let pk = Keys::generate().public_key();
    let a1 = Filter::new().kind(Kind::from(0u16)).author(pk);
    let a2 = Filter::new().kind(Kind::from(0u16)).author(pk);
    let b = Filter::new().kind(Kind::from(9u16)).author(pk);

    // Same filter content -> identical id, so resubscribe replaces rather than
    // accumulating a fresh random-id subscription.
    assert_eq!(stable_subscription_id(&a1), stable_subscription_id(&a2));
    assert_ne!(stable_subscription_id(&a1), stable_subscription_id(&b));
}

// ── assert_relay_accepted unit tests ────────────────────────────────────────

fn output_with(success: &[&str], failed: &[(&str, &str)]) -> Output<EventId> {
    let mut out: Output<EventId> = Output {
        val: EventId::all_zeros(),
        success: Default::default(),
        failed: Default::default(),
    };
    for url in success {
        out.success.insert(RelayUrl::parse(url).unwrap());
    }
    for (url, reason) in failed {
        out.failed
            .insert(RelayUrl::parse(url).unwrap(), reason.to_string());
    }
    out
}

#[test]
fn accepted_when_any_relay_succeeds() {
    let out = output_with(&["wss://ok.relay"], &[("wss://bad.relay", "blocked")]);
    assert!(assert_relay_accepted(&out, None).is_ok());
}

#[test]
fn duplicate_response_counts_as_already_accepted() {
    let out = output_with(&[], &[("wss://ok.relay", "duplicate: already have event")]);
    assert!(assert_relay_accepted(&out, None).is_ok());
}

#[test]
fn rejected_surfaces_relay_reason() {
    let out = output_with(&[], &[("wss://nip29.relay", "blocked: unknown member")]);
    let err = assert_relay_accepted(&out, None).unwrap_err().to_string();
    assert!(err.contains("blocked: unknown member"), "got: {err}");
}

#[test]
fn no_accept_no_reason_reports_timeout() {
    // Every relay silent: send_event resolved Ok but no OK,true ever arrived.
    let out = output_with(&[], &[]);
    let err = assert_relay_accepted(&out, None).unwrap_err().to_string();
    assert!(err.contains("no relay accepted"), "got: {err}");
}
