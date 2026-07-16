use super::*;

fn chans<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

fn seeded(working: bool, title: &str) -> StatusReconciler {
    let mut policy = StatusReconciler::new(90, 30);
    let out = policy.on_session_started(
        "pk1",
        "laptop",
        "coder",
        ".",
        chans(["room"]),
        working,
        true,
        title,
        0,
    );
    assert_eq!(out.effects.len(), 1);
    policy
}

fn published(effects: &[StatusEffect]) -> Option<(&Status, PublishReason)> {
    effects.iter().find_map(|effect| match effect {
        StatusEffect::Publish { status, reason } => Some((status, *reason)),
        StatusEffect::Expire { .. } => None,
    })
}

#[test]
fn identical_content_is_deduped() {
    let mut policy = seeded(true, "Old");
    assert_eq!(
        published(&policy.on_title_set("pk1", "New", 0).effects)
            .unwrap()
            .1,
        PublishReason::Changed
    );
    assert!(policy.on_title_set("pk1", "New", 0).effects.is_empty());
}

#[test]
fn manual_title_change_publishes_title_without_activity() {
    let mut policy = seeded(true, "Old");
    let changed = policy.on_title_set("pk1", "Testing status updates", 0);
    let (status, reason) = published(&changed.effects).unwrap();
    assert_eq!(reason, PublishReason::Changed);
    assert_eq!(status.title, "Testing status updates");
    assert!(status.activity.is_empty());
}

#[test]
fn turn_end_is_idle_and_channel_change_updates_tags() {
    let mut policy = seeded(true, "T");
    let ended = policy.on_turn_end("pk1", 0);
    let (status, reason) = published(&ended.effects).unwrap();
    assert_eq!(reason, PublishReason::Changed);
    assert_eq!(status.state, crate::session_state::SessionState::Idle);
    assert!(status.activity.is_empty());

    let changed = policy.on_channels_changed("pk1", chans(["other"]), 0);
    assert_eq!(published(&changed.effects).unwrap().0.channels, ["other"]);
}

#[test]
fn refresh_bucket_rearms_without_content_change() {
    let mut policy = seeded(true, "T");
    let refresh = policy.on_tick("pk1", true, 30);
    let (status, reason) = published(&refresh.effects).unwrap();
    assert_eq!(reason, PublishReason::Refreshed);
    assert_eq!(status.expires_at, Some(120));
    assert!(policy.on_tick("pk1", true, 45).effects.is_empty());
}

#[test]
fn end_and_revoke_publish_offline_status() {
    let mut ended_policy = seeded(true, "T");
    let ended = ended_policy.on_session_ended("pk1", 200);
    let status = published(&ended.effects).unwrap().0;
    assert_eq!(status.state, crate::session_state::SessionState::Offline);
    assert_eq!(status.expires_at, Some(200));
    assert_eq!(status.channels, ["room"]);

    let mut revoked_policy = seeded(true, "T");
    let revoked = revoked_policy.on_session_revoked("pk1", 123);
    let StatusEffect::Expire { status } = &revoked.effects[0] else {
        panic!("expected explicit expiration")
    };
    assert_eq!(status.expires_at, Some(123));
    assert_eq!(status.state, crate::session_state::SessionState::Offline);
}

#[test]
fn forgetting_and_unknown_sessions_are_noops() {
    let mut policy = seeded(false, "T");
    policy.forget_session("pk1");
    assert!(policy.on_turn_start("pk1", 10).effects.is_empty());
    assert!(StatusReconciler::new(90, 30)
        .on_turn_start("ghost", 10)
        .effects
        .is_empty());
}
