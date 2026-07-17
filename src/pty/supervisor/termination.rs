use anyhow::{Context, Result};
use std::time::Duration;

pub(super) fn terminate_child_confirmed(
    child: &mut (dyn portable_pty::Child + Send + Sync),
) -> Result<portable_pty::ExitStatus> {
    if let Some(status) = child.try_wait()? {
        return Ok(status);
    }
    let pid = child.process_id();
    if let Err(signal_error) = child.kill() {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        return Err(signal_error).context("signalling PTY child with SIGHUP");
    }
    if let Some(status) = wait_for_child(child, Duration::from_millis(500))? {
        return Ok(status);
    }
    let pid = pid.context("PTY child has no process id for forced termination")?;
    let pid = i32::try_from(pid).context("PTY child pid overflow")?;
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid),
        Some(nix::sys::signal::Signal::SIGKILL),
    )
    .with_context(|| format!("forcing PTY child pid {pid} to exit"))?;
    wait_for_child(child, Duration::from_secs(2))?.context("PTY child did not exit after SIGKILL")
}

fn wait_for_child(
    child: &mut (dyn portable_pty::Child + Send + Sync),
    timeout: Duration,
) -> Result<Option<portable_pty::ExitStatus>> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if std::time::Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    #[test]
    fn termination_confirms_exit_when_child_ignores_sighup() {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new("/bin/sh");
        command.args(["-c", "trap '' HUP; exec sleep 60"]);
        let mut child = pair.slave.spawn_command(command).unwrap();
        std::thread::sleep(Duration::from_millis(100));

        let status = terminate_child_confirmed(child.as_mut()).unwrap();

        assert!(!status.success());
        assert!(child.try_wait().unwrap().is_some());
    }
}
