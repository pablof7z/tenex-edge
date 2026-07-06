//! Channel target evidence for `probe validate`.

mod evidence;
mod readiness;

pub(super) use evidence::{channel_evidence, push_channel_check};
