//! Shared test harness: spin up a real in-memory relay via `nak serve`.

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct TestRelay {
    child: Child,
    pub url: String,
}

fn nak_bin() -> PathBuf {
    if let Ok(p) = std::env::var("NAK") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidate = PathBuf::from(&home).join("go/bin/nak");
    if candidate.exists() {
        return candidate;
    }
    PathBuf::from("nak") // rely on PATH
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

impl TestRelay {
    pub fn start() -> Self {
        let port = free_port();
        let child = Command::new(nak_bin())
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--quiet")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn `nak serve` (is nak installed?)");

        // Wait for the relay to accept TCP connections.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            if Instant::now() > deadline {
                panic!("nak serve did not come up on port {port}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        TestRelay {
            child,
            url: format!("ws://localhost:{port}"),
        }
    }
}

impl Drop for TestRelay {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
