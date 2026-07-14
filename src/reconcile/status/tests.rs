use super::*;
use trellis_core::{ResourceCommandCause, ResourceCommandKind};
use trellis_testing::ResourceLedger;

fn chans<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Seed a live session (busy, one channel) and return the reconciler + ledger.
fn seeded(
    working: bool,
    title: &str,
    activity: &str,
    channels: BTreeSet<String>,
    now: u64,
) -> (StatusReconciler, ResourceLedger<StatusCommand>) {
    let mut r = StatusReconciler::new(90, 30);
    let mut ledger = ResourceLedger::new();
    let out = r
        .on_session_started(
            "pk1", "laptop", "coder", ".", channels, working, title, activity, now,
        )
        .unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();
    // Startup always opens.
    assert_eq!(publishes(&out.effects).len(), 1);
    (r, ledger)
}

fn publishes(effects: &[StatusEffect]) -> Vec<(&Status, PublishReason)> {
    effects
        .iter()
        .filter_map(|e| match e {
            StatusEffect::Publish { status, reason } => Some((status, *reason)),
            _ => None,
        })
        .collect()
}

/// HEADLINE: the same session state committed twice — the SECOND commit emits no
/// publish command. This is the dedup the old five-trigger path never had.
#[test]
fn identical_state_commit_is_deduped() {
    let (mut r, mut ledger) = seeded(true, "T", "", chans(["room"]), 100);

    // First distill: activity changes → exactly one publish.
    let first = r.on_distill("pk1", "T", "reading", 100).unwrap();
    ledger.apply_result(&first.result);
    r.assert_oracle().unwrap();
    assert_eq!(publishes(&first.effects).len(), 1);

    // Committing the IDENTICAL state again (same title/activity, same tick bucket)
    // must emit NOTHING — the graph change-detection swallows it.
    let second = r.on_distill("pk1", "T", "reading", 100).unwrap();
    ledger.apply_result(&second.result);
    r.assert_oracle().unwrap();
    assert!(
        second.effects.is_empty(),
        "identical re-commit must not publish: {:?}",
        second.effects
    );

    ledger.assert_all_resources_have_owner().unwrap();
    ledger.assert_no_duplicate_close().unwrap();
}

/// A distill that changes activity publishes exactly one status with the new
/// content, and the receipt attributes the command to the `activity` input.
#[test]
fn distill_change_publishes_and_attributes_to_activity() {
    let (mut r, mut ledger) = seeded(true, "T", "", chans(["room"]), 100);
    let activity_node = r.activity_input("pk1").unwrap();

    let out = r.on_distill("pk1", "T", "compiling", 100).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1, "one publish for the content change");
    assert_eq!(pubs[0].0.activity, "compiling");
    assert_eq!(pubs[0].1, PublishReason::Changed);

    let why = r.why_command("pk1").expect("a command was emitted for pk1");
    assert_eq!(why.kind, ResourceCommandKind::Replace);
    assert!(
        why.input_causes.contains(&activity_node),
        "publish attributed to the activity input: {:?}",
        why.input_causes
    );
}

#[test]
fn manual_title_change_publishes_title_without_clearing_activity() {
    let (mut r, mut ledger) = seeded(true, "Old", "checking logs", chans(["room"]), 100);

    let out = r
        .on_title_set("pk1", "Testing status updates and awareness", 100)
        .unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1, "one publish for the title change");
    assert_eq!(pubs[0].0.title, "Testing status updates and awareness");
    assert_eq!(pubs[0].0.activity, "checking logs");
    assert_eq!(pubs[0].1, PublishReason::Changed);
}

/// Turn-end flips the session to idle: exactly one publish, activity cleared.
#[test]
fn turn_end_flips_to_idle_one_publish() {
    let (mut r, mut ledger) = seeded(true, "T", "writing", chans(["room"]), 100);

    let out = r.on_turn_end("pk1", 100).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1);
    assert!(!pubs[0].0.busy, "idle after turn-end");
    assert_eq!(pubs[0].0.activity, "", "idle clears the live activity line");
    assert_eq!(pubs[0].1, PublishReason::Changed);
}

/// Ending a session publishes a final idle status that keeps the session visible
/// for the normal NIP-40 TTL window.
#[test]
fn session_end_emits_idle_status_with_full_ttl() {
    let (mut r, mut ledger) = seeded(true, "T", "busy", chans(["room"]), 100);

    let out = r.on_session_ended("pk1", 200).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1, "session-end emits one final status");
    assert_eq!(pubs[0].0.expires_at, Some(290), "normal TTL is retained");
    assert!(!pubs[0].0.busy);
    assert_eq!(pubs[0].0.activity, "", "activity cleared on teardown");
    assert_eq!(
        pubs[0].0.channels,
        vec!["room".to_string()],
        "final idle status keeps the last-known h tags"
    );
    assert_eq!(pubs[0].1, PublishReason::Changed);

    ledger.assert_all_resources_have_owner().unwrap();
    ledger.assert_no_duplicate_close().unwrap();
}

#[test]
fn session_end_rearms_ttl_even_when_already_idle() {
    let (mut r, mut ledger) = seeded(false, "T", "", chans(["room"]), 100);

    let out = r.on_session_ended("pk1", 100).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1);
    assert_eq!(pubs[0].0.expires_at, Some(190));
    assert!(!pubs[0].0.busy);
    assert_eq!(pubs[0].1, PublishReason::Refreshed);
}

#[test]
fn operator_revoke_expires_status_now_and_closes_the_session_row() {
    let (mut r, mut ledger) = seeded(true, "T", "busy", chans(["room"]), 100);

    let out = r.on_session_revoked("pk1", 123).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    assert_eq!(out.effects.len(), 1);
    let StatusEffect::Expire { status } = &out.effects[0] else {
        panic!("operator revoke must emit an expiring status")
    };
    assert_eq!(status.expires_at, Some(123));
    assert_eq!(status.channels, vec!["room".to_string()]);
    assert!(!status.busy);
    assert!(status.activity.is_empty());
    assert!(r.state_rows().is_empty());
    ledger.assert_no_duplicate_close().unwrap();
}

#[test]
fn forgetting_stale_session_closes_local_graph_without_publish() {
    let (mut r, ledger) = seeded(false, "T", "", chans(["room"]), 100);

    r.forget_session("pk1").unwrap();
    r.assert_oracle().unwrap();

    assert!(r.state_rows().is_empty());
    let why = r.why_command("pk1").expect("a close was emitted for pk1");
    assert_eq!(why.kind, ResourceCommandKind::Close);
    assert!(
        matches!(why.cause, ResourceCommandCause::ScopeClosed { .. }),
        "close cause names the session scope teardown: {:?}",
        why.cause
    );
    ledger.assert_all_resources_have_owner().unwrap();
    ledger.assert_no_duplicate_close().unwrap();
}

/// Leaving a channel corrects the derived `h`-tag set with a single publish
/// (fixes the stale-h-tag bug — the old path never retracted).
#[test]
fn channel_leave_corrects_h_tags() {
    let (mut r, mut ledger) = seeded(true, "T", "x", chans(["a", "b"]), 100);

    let out = r.on_channels_changed("pk1", chans(["a"]), 100).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();

    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1);
    assert_eq!(
        pubs[0].0.channels,
        vec!["a".to_string()],
        "h-tag set corrected to the remaining channel"
    );
    assert_eq!(pubs[0].1, PublishReason::Changed);
}

/// A TTL tick that crosses a refresh bucket re-arms the NIP-40 window WITHOUT a
/// content change — a distinct `Refresh` reason — and an in-bucket tick is a
/// no-op. Proves the SINGLE refresh cadence subsumes both old timers.
#[test]
fn tick_rearms_without_content_change_and_is_the_single_path() {
    let (mut r, mut ledger) = seeded(true, "T", "x", chans(["room"]), 0);

    // Cross into the next 30s refresh bucket → one refresh (content unchanged).
    let out = r.on_tick("pk1", 30).unwrap();
    ledger.apply_result(&out.result);
    r.assert_oracle().unwrap();
    let pubs = publishes(&out.effects);
    assert_eq!(pubs.len(), 1);
    assert_eq!(pubs[0].1, PublishReason::Refreshed);
    assert_eq!(pubs[0].0.expires_at, Some(120), "TTL re-armed to now + ttl");
    let why = r.why_command("pk1").unwrap();
    assert_eq!(why.kind, ResourceCommandKind::Refresh);

    // Same bucket again → nothing (no unconditional republish).
    let again = r.on_tick("pk1", 45).unwrap();
    ledger.apply_result(&again.result);
    r.assert_oracle().unwrap();
    assert!(again.effects.is_empty(), "in-bucket tick is a no-op");

    ledger.assert_all_resources_have_owner().unwrap();
    ledger.assert_no_duplicate_close().unwrap();
}

/// An unknown-session method call is a clean no-op that still returns a receipt.
#[test]
fn unknown_session_is_a_noop() {
    let mut r = StatusReconciler::new(90, 30);
    let out = r.on_turn_start("ghost", 10).unwrap();
    assert!(out.effects.is_empty());
    r.assert_oracle().unwrap();
}
