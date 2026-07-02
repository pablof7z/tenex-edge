//! Agent turn-context assembly shared by daemon RPCs and hook tests.
//!
//! The CLI owns hook I/O and rendering commands. The daemon owns turn state.
//! This module owns the shared text/audit assembly used between them.

mod audit;
mod check;
mod reads;
mod start;

pub(crate) use audit::{turn_check_audit, turn_start_audit};
pub(crate) use check::assemble_turn_check_context;
pub(crate) use start::assemble_turn_start_context;

#[cfg(test)]
mod tests;
