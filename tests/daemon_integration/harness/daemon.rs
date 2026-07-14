use super::Home;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// Stop the daemon through version skew and wait for its socket to disappear.
pub(crate) fn stop_daemon(home: &Home) {
    if let Ok(stream) = UnixStream::connect(home.sock()) {
        let mut writer = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let _ = writeln!(
            writer,
            "{}",
            serde_json::json!({"protocol": u32::MAX, "client_version": "t"})
        );
        let mut welcome = String::new();
        let _ = reader.read_line(&mut welcome);
        let _ = writeln!(writer, "{}", serde_json::json!({"protocol": u32::MAX}));
        let mut response = String::new();
        let _ = reader.read_line(&mut response);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && home.sock().exists() {
        std::thread::sleep(Duration::from_millis(25));
    }
    let _ = std::fs::remove_file(home.dir.path().join("daemon.lock"));
}
