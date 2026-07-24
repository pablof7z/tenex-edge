use super::*;

#[test]
fn agent_chunks_stream_into_one_entry() {
    let mut t = Transcript::new();
    t.apply(DeployEvent::Agent("Hel".into()));
    t.apply(DeployEvent::Agent("lo".into()));
    assert_eq!(t.entries, vec![Entry::Agent("Hello".into())]);
    assert_eq!(t.status, DeployStatus::Working);
}

#[test]
fn activity_breaks_agent_streaming() {
    let mut t = Transcript::new();
    t.apply(DeployEvent::Agent("run".into()));
    t.apply(DeployEvent::Activity("bash: apt install".into()));
    t.apply(DeployEvent::Agent("done".into()));
    assert_eq!(
        t.entries,
        vec![
            Entry::Agent("run".into()),
            Entry::Activity("bash: apt install".into()),
            Entry::Agent("done".into()),
        ]
    );
}

#[test]
fn thoughts_and_agent_text_do_not_merge() {
    let mut t = Transcript::new();
    t.apply(DeployEvent::Thought("hmm".into()));
    t.apply(DeployEvent::Agent("answer".into()));
    assert_eq!(
        t.entries,
        vec![Entry::Thought("hmm".into()), Entry::Agent("answer".into())]
    );
}

#[test]
fn error_sets_failed_and_is_sticky() {
    let mut t = Transcript::new();
    t.apply(DeployEvent::Error("child exited".into()));
    assert_eq!(t.status, DeployStatus::Failed("child exited".into()));
    // A later turn-ended must not clear a failure.
    t.apply(DeployEvent::TurnEnded);
    assert_eq!(t.status, DeployStatus::Failed("child exited".into()));
}

#[test]
fn turn_ended_goes_idle_when_not_terminal() {
    let mut t = Transcript::new();
    t.apply(DeployEvent::Agent("working".into()));
    t.apply(DeployEvent::TurnEnded);
    assert_eq!(t.status, DeployStatus::Idle);
}

#[test]
fn relay_online_is_terminal_over_permission_toggles() {
    let mut t = Transcript::new();
    t.relay_online();
    assert_eq!(t.status, DeployStatus::RelayOnline);
    t.set_awaiting_permission(true);
    assert_eq!(t.status, DeployStatus::RelayOnline);
}

#[test]
fn permission_toggle_moves_between_working_and_awaiting() {
    let mut t = Transcript::new();
    t.set_awaiting_permission(true);
    assert_eq!(t.status, DeployStatus::AwaitingPermission);
    t.set_awaiting_permission(false);
    assert_eq!(t.status, DeployStatus::Working);
}
