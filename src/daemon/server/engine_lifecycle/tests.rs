use super::{pid_alive, revive_decision};

#[test]
fn nonpositive_pid_is_never_alive() {
    // Defect #3: a synth ACP pid of 0 (`kill(0)` hits the caller's own group)
    // and negative pids (`kill(-n)` hits a whole group) must read as NOT live,
    // so a dead ACP session is never treated as an immortal ghost.
    assert!(!pid_alive(0));
    assert!(!pid_alive(-1));
}

#[test]
fn dead_pid_is_never_revived() {
    assert!(!revive_decision(false, None));
    assert!(!revive_decision(false, Some(true)));
    assert!(!revive_decision(false, Some(false)));
}

#[test]
fn exec_session_revives_on_pid_alone() {
    // No PTY socket => PID liveness is authoritative.
    assert!(revive_decision(true, None));
}

#[test]
fn live_pid_with_live_pty_is_revived() {
    assert!(revive_decision(true, Some(true)));
}

#[test]
fn live_pid_with_dead_pty_is_not_revived() {
    // Guards against PID recycling: the process at `child_pid` is alive but
    // its supervisor socket is gone, so it is not our session.
    assert!(!revive_decision(true, Some(false)));
}
