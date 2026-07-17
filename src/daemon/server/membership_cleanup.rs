//! Explicit destructive session revocation.
//!
//! Timed runtime and standing transitions belong to `managed_lifecycle`; this
//! module exists only for the operator-authorized forget boundary.

use super::*;

mod revoke;
pub(super) use revoke::recorded_channels;
pub(in crate::daemon::server) use revoke::remove_revoked_session_memberships;
