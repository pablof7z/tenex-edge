use nix::errno::Errno;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub(super) fn isolate(command: &mut tokio::process::Command) {
    command.process_group(0);
}

pub(super) fn signal(group: i32) -> std::io::Result<bool> {
    match kill(Pid::from_raw(-group), Some(Signal::SIGKILL)) {
        Ok(()) => Ok(true),
        Err(Errno::ESRCH) => Ok(false),
        Err(error) => Err(std::io::Error::from_raw_os_error(error as i32)),
    }
}

pub(super) async fn wait_exit(group: i32) -> std::io::Result<()> {
    for _ in 0..200 {
        match kill(Pid::from_raw(-group), None) {
            Err(Errno::ESRCH) => return Ok(()),
            // macOS can report EPERM while a just-killed process group is
            // transitioning through reap. It is not exit confirmation, so
            // keep polling until ESRCH or the deadline.
            Ok(()) | Err(Errno::EPERM) => {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await
            }
            Err(error) => return Err(std::io::Error::from_raw_os_error(error as i32)),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("timed out waiting for RPC harness process group {group} to exit"),
    ))
}
