//! Relay-assist modal: a Mosaico-owned structured agent panel.
//!
//! When the operator asks for help provisioning a relay, this drives a selected
//! ACP / app-server harness child (bootstrap-owned, outside the daemon) and
//! renders a normalized transcript in Mosaico's own visual language — routing
//! permission prompts to the human and polling the target URL until a NIP-29
//! relay is reachable there. Mosaico does not run or supervise the relay; it
//! helps the operator stand one up and detects success.
//!
//! NOTE: the transcript reducer, event decoders, and harness resolution are
//! unit-tested. The live driver (`driver.rs`) follows the `acp_smoke` template
//! but is not exercised against a running harness in CI.

mod decode;
mod driver;
mod harness;
mod render;
mod session;
mod transcript;

use std::sync::mpsc;

pub(super) use harness::{can_assist, resolve};
pub(super) use render::draw as draw_modal;
pub(super) use session::{DeployOutcome, DeploySession};

/// A human-routed permission request surfaced from the harness child. Carries
/// its own responder so concurrent requests never cross wires.
pub(in crate::cli::install::onboarding) struct PermissionAsk {
    pub summary: String,
    pub options: Vec<PermissionOption>,
    pub respond: mpsc::Sender<Option<String>>,
}

pub(in crate::cli::install::onboarding) struct PermissionOption {
    pub id: String,
    pub label: String,
    pub allow: bool,
}
