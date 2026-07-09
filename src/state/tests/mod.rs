//! Persistence-foundation tests: canonical session identity, NIP-01 replacement,
//! NIP-40 status liveness, and unique-pubkey-per-channel membership.
//!
//! Split by theme into sibling files to stay under the repo's per-file LOC
//! ceiling; shared fixtures live here.

use super::*;

fn reg(harness: &str, ext: &str, channel: &str) -> RegisterSession {
    RegisterSession {
        harness: harness.into(),
        external_id_kind: "harness_session".into(),
        external_id: ext.into(),
        agent_pubkey: "pk-agent".into(),
        agent_slug: "agent".into(),
        channel_h: channel.into(),
        child_pid: Some(42),
        transcript_path: Some("/t/x.jsonl".into()),
        resume_id: String::new(),
        now: 1000,
    }
}

mod channels_tree;
mod identities_and_roots;
mod inbox_outbox;
mod nip01_events;
mod retention;
mod session_identity;
mod status_membership;
