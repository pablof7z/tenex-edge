use super::{AutoReplyTracker, PendingAutoReply};

fn pending(channel: &str, event: &str, requester: &str) -> PendingAutoReply {
    PendingAutoReply {
        channel_h: channel.to_string(),
        trigger_event_id: event.to_string(),
        requester_pubkey: requester.to_string(),
    }
}

#[test]
fn armed_turn_with_no_publish_yields_pending() {
    let mut t = AutoReplyTracker::default();
    t.arm("s1", "chan", "evt1", "requester-pk");
    assert_eq!(t.take("s1"), Some(pending("chan", "evt1", "requester-pk")));
}

#[test]
fn explicit_publish_cancels_auto_reply() {
    let mut t = AutoReplyTracker::default();
    t.arm("s1", "chan", "evt1", "requester-pk");
    t.note_explicit_publish("s1");
    assert_eq!(t.take("s1"), None);
}

#[test]
fn non_injected_turn_has_nothing_to_publish() {
    let mut t = AutoReplyTracker::default();
    assert_eq!(t.take("s-never-armed"), None);
}

#[test]
fn newest_mention_supersedes_earlier_unanswered() {
    let mut t = AutoReplyTracker::default();
    t.arm("s1", "chan", "evt1", "old-requester");
    t.arm("s1", "chan2", "evt2", "new-requester");
    assert_eq!(
        t.take("s1"),
        Some(pending("chan2", "evt2", "new-requester"))
    );
}

#[test]
fn explicit_publish_disables_future_auto_reply_for_session() {
    let mut t = AutoReplyTracker::default();
    t.note_explicit_publish("s1");

    assert!(!t.arm("s1", "chan", "evt1", "requester-pk"));
    assert_eq!(t.take("s1"), None);

    assert!(t.arm("s2", "chan", "evt2", "requester-pk"));
    assert_eq!(t.take("s2"), Some(pending("chan", "evt2", "requester-pk")));
}

#[test]
fn take_is_one_shot() {
    let mut t = AutoReplyTracker::default();
    t.arm("s1", "chan", "evt1", "requester-pk");
    assert!(t.take("s1").is_some());
    assert!(t.take("s1").is_none());
}
