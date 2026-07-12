//! Process-liveness helper shared across the daemon (single source of truth for
//! the `pid_alive` check — defect #17).

/// Whether a process id is still alive, via a null signal (`kill(pid, 0)`).
///
/// Guards non-positive pids (defect #3/#389): `kill(0, ...)` targets the
/// CALLER's process group and `kill(-n, ...)` a whole group, both of which
/// spuriously succeed. A synth ACP pid of `0` (no reported child pid) must read
/// as NOT live, so a dead session is never treated as an immortal ghost.
pub(crate) fn pid_alive(pid: i32) -> bool {
    pid > 0 && nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok()
}

#[cfg(test)]
mod tests {
    use super::pid_alive;

    #[test]
    fn current_pid_is_alive() {
        assert!(pid_alive(std::process::id() as i32));
    }

    #[test]
    fn nonpositive_pid_is_never_alive() {
        // Defect #3: a synth ACP pid of 0 (`kill(0)` hits the caller's own group)
        // and negative pids (`kill(-n)` hits a whole group) must read as NOT live.
        assert!(!pid_alive(0));
        assert!(!pid_alive(-1));
    }
}
