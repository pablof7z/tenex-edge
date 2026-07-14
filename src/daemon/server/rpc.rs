//! RPC handler families extracted from `server.rs` (issue #12, EPIC-server-001).
//!
//! Each submodule owns one cohesive handler family. Handlers are `pub(super)`
//! inside their submodule and re-exported here as `pub` so the dispatch
//! table in `server.rs` can call them as `rpc::rpc_*`.

pub(super) mod channel_members;
pub(super) mod pty_supervisor;

pub use channel_members::{
    rpc_channel_add_member, rpc_channel_members, rpc_channel_remove_member, rpc_root_channels,
};
pub(super) use pty_supervisor::rpc_pty_supervisor_exit;
