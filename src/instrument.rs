//! Retrospective instrumentation host-boundary helpers.
//!
//! Storage ledgers are pure and never read the clock. This module provides the
//! host-side glue that records provenance without blocking the hot path.

use crate::state::receipts::NewReceipt;
use crate::state::Store;

/// Host wall clock in unix milliseconds, read at the boundary (ledgers never do).
pub fn now_millis() -> i64 {
    crate::util::now_millis() as i64
}

/// Record one provenance receipt without blocking its owning operation.
pub fn record_receipt(store: &Store, row: NewReceipt) {
    if let Err(error) = store.record_receipt(&row) {
        tracing::warn!(
            surface = %row.surface,
            error = %error,
            "record_receipt failed — operation not instrumented"
        );
    }
}
