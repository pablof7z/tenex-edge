use super::*;

fn view() -> StatuslineView {
    StatuslineView {
        agent: "amber-claude".into(),
        host: "Kubrick's Mac".into(),
        session_id: "some-long-uuid".into(),
        work_root: "mosaico".into(),
        channel: "41yh4c028b76a".into(),
        channel_title: "support".into(),
        member_count: 4,
        is_member: true,
        working: true,
        title: "Refactoring the inbox".into(),
        activity: "writing tests".into(),
        error: None,
    }
}

#[test]
fn renders_identity_root_session_title_status() {
    assert_eq!(
        render_statusline(&view(), false),
        "amber-claude mosaico support [Refactoring the inbox] [writing tests]"
    );
}

#[test]
fn busy_with_no_activity_shows_working() {
    let mut v = view();
    v.activity.clear();
    assert!(render_statusline(&v, false).ends_with("[working]"));
}

#[test]
fn idle_shows_idle() {
    let mut v = view();
    v.working = false;
    assert!(render_statusline(&v, false).ends_with("[idle]"));
}

#[test]
fn empty_channel_title_omits_title_segment() {
    let mut v = view();
    v.channel_title.clear();
    let rendered = render_statusline(&v, false);
    assert!(!rendered.contains("[]"));
    assert!(rendered.contains("[writing tests]"));
}

#[test]
fn membership_gap_is_loud() {
    let mut v = view();
    v.is_member = false;
    assert!(render_statusline(&v, false).contains("⚠ not in channel support"));
    v.member_count = 0;
    assert!(!render_statusline(&v, false).contains("not in channel"));
}

#[test]
fn truncates_long_channel_title() {
    let mut v = view();
    v.channel_title = "x".repeat(100);
    assert!(render_statusline(&v, false).contains('…'));
}

#[test]
fn truncates_long_activity() {
    let mut v = view();
    v.activity = "y".repeat(100);
    assert!(render_statusline(&v, false).contains('…'));
}
