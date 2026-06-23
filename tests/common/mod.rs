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

/// Path to the croissant relay binary — `nak serve` does NOT implement NIP-29
/// group semantics (9007/9002 creates, 39001 admin reflection), so any test
/// that owns groups or mints subgroups must run against croissant instead.
/// Override with `$CROISSANT_BIN`.
fn croissant_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CROISSANT_BIN") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Work/croissant/croissant")
}

impl TestRelay {
    /// Spawn a real croissant relay (NIP-29 aware) on an ephemeral port with an
    /// isolated data dir. Use for daemon tests that exercise group ownership /
    /// subgroup minting.
    pub fn start_croissant() -> Self {
        let port = free_port();
        let bin = croissant_bin();
        assert!(
            bin.exists(),
            "croissant binary not found at {} (set $CROISSANT_BIN)",
            bin.display()
        );
        let data = std::env::temp_dir().join(format!("croissant-test-{port}"));
        let _ = std::fs::remove_dir_all(&data);
        let child = Command::new(&bin)
            .env("PORT", port.to_string())
            .env("HOST", "127.0.0.1")
            .env("DATAPATH", &data)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn croissant");

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            if Instant::now() > deadline {
                panic!("croissant did not come up on port {port}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        TestRelay {
            child,
            url: format!("ws://127.0.0.1:{port}"),
        }
    }
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
