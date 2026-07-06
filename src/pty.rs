//! Portable PTY supervisor and client surface for reattachable agent sessions.

mod client;
mod launch;
mod meta;
mod supervisor;

pub use client::{attach, inject, is_live, kill, list, resize};
pub use launch::{spawn_session, SpawnSessionArgs};
pub use meta::{read_all_metadata, session_dir, session_socket, write_metadata, LaunchMetadata};
pub use supervisor::{run_supervisor, SupervisorArgs};
