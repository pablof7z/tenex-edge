//! Background daemon tasks extracted from `server.rs` (issue #12,
//! EPIC-server-001).
//!
//! Each submodule owns one long-lived `tokio::spawn` loop driving a piece of
//! daemon housekeeping. The spawn entry points are `pub(super)` and re-exported
//! here so the accept-loop bootstrap in `server.rs` can start them by name.
//!
//! Pure function movement — behavior is byte-identical to the pre-split file.

mod pruner;
mod trellis_oracle;

pub(super) use pruner::spawn_pruner;
pub(super) use trellis_oracle::spawn_trellis_oracle_sampler;
