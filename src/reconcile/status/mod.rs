//! Per-session status publication policy.
//!
//! Mosaico owns product state and change detection. The host submits emitted
//! effects through NMP's durable write plane.

mod command;
mod status_build;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use crate::domain::Status;

pub use command::{StatusCommand, StatusOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishReason {
    Opened,
    Changed,
    Refreshed,
}

impl PublishReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::Changed => "changed",
            Self::Refreshed => "refreshed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatusEffect {
    Publish {
        status: Status,
        reason: PublishReason,
    },
    Expire {
        status: Status,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StaticInfo {
    host: String,
    slug: String,
    rel_cwd: String,
    dispatch_event: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SessionStatus {
    info: StaticInfo,
    channels: BTreeSet<String>,
    live: bool,
    working: bool,
    automatic_delivery: bool,
    title: String,
    arm: u64,
}

#[derive(Clone)]
pub struct StatusReconciler {
    ttl_secs: u64,
    refresh_secs: u64,
    sessions: BTreeMap<String, SessionStatus>,
    revision: u64,
}

impl StatusReconciler {
    pub fn new(ttl_secs: u64, refresh_secs: u64) -> Self {
        Self {
            ttl_secs: ttl_secs.max(1),
            refresh_secs: refresh_secs.max(1),
            sessions: BTreeMap::new(),
            revision: 0,
        }
    }

    pub fn for_ttl(ttl: Duration) -> Self {
        Self::new(ttl.as_secs(), crate::domain::HEARTBEAT_SECS)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_session_started(
        &mut self,
        pubkey: &str,
        host: &str,
        slug: &str,
        rel_cwd: &str,
        channels: BTreeSet<String>,
        working: bool,
        automatic_delivery: bool,
        title: &str,
        now: u64,
    ) -> StatusOutcome {
        self.on_session_started_with_dispatch(
            pubkey,
            host,
            slug,
            rel_cwd,
            channels,
            working,
            automatic_delivery,
            title,
            None,
            now,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_session_started_with_dispatch(
        &mut self,
        pubkey: &str,
        host: &str,
        slug: &str,
        rel_cwd: &str,
        channels: BTreeSet<String>,
        working: bool,
        automatic_delivery: bool,
        title: &str,
        dispatch_event: Option<String>,
        now: u64,
    ) -> StatusOutcome {
        if self.sessions.contains_key(pubkey) {
            return self.empty_outcome(Some(pubkey));
        }
        let state = SessionStatus {
            info: StaticInfo {
                host: host.to_string(),
                slug: slug.to_string(),
                rel_cwd: rel_cwd.to_string(),
                dispatch_event,
            },
            channels,
            live: true,
            working,
            automatic_delivery,
            title: title.to_string(),
            arm: now / self.refresh_secs,
        };
        let command = command_of(pubkey, &state);
        self.sessions.insert(pubkey.to_string(), state);
        let revision = self.bump_revision();
        StatusOutcome {
            effects: vec![StatusEffect::Publish {
                status: status_build::to_status(&command, self.ttl_secs, now, false),
                reason: PublishReason::Opened,
            }],
            revision,
            pubkey: Some(pubkey.to_string()),
        }
    }

    pub fn on_turn_start(&mut self, id: &str, now: u64) -> StatusOutcome {
        self.mutate(id, now, |state| state.working = true)
    }

    pub fn on_turn_end(&mut self, id: &str, now: u64) -> StatusOutcome {
        self.mutate(id, now, |state| state.working = false)
    }

    pub fn on_title_set(&mut self, id: &str, title: &str, now: u64) -> StatusOutcome {
        self.mutate(id, now, |state| state.title = title.to_string())
    }

    pub fn on_channels_changed(
        &mut self,
        id: &str,
        channels: BTreeSet<String>,
        now: u64,
    ) -> StatusOutcome {
        self.mutate(id, now, move |state| state.channels = channels)
    }

    pub fn on_tick(&mut self, id: &str, automatic_delivery: bool, now: u64) -> StatusOutcome {
        self.mutate(id, now, |state| {
            state.automatic_delivery = automatic_delivery
        })
    }

    pub fn on_session_ended(&mut self, id: &str, now: u64) -> StatusOutcome {
        let final_arm = now / self.refresh_secs + 1;
        self.mutate_with_arm(id, now, final_arm, |state| {
            state.live = false;
            state.working = false;
        })
    }

    pub fn on_session_revoked(&mut self, id: &str, now: u64) -> StatusOutcome {
        let Some(state) = self.sessions.remove(id) else {
            return self.empty_outcome(Some(id));
        };
        let command = command_of(id, &state);
        let revision = self.bump_revision();
        StatusOutcome {
            effects: vec![StatusEffect::Expire {
                status: status_build::to_status(&command, self.ttl_secs, now, true),
            }],
            revision,
            pubkey: Some(id.to_string()),
        }
    }

    pub fn forget_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    fn mutate(
        &mut self,
        id: &str,
        now: u64,
        update: impl FnOnce(&mut SessionStatus),
    ) -> StatusOutcome {
        self.mutate_with_arm(id, now, now / self.refresh_secs, update)
    }

    fn mutate_with_arm(
        &mut self,
        id: &str,
        now: u64,
        arm: u64,
        update: impl FnOnce(&mut SessionStatus),
    ) -> StatusOutcome {
        let Some(state) = self.sessions.get_mut(id) else {
            return self.empty_outcome(Some(id));
        };
        let previous = command_of(id, state);
        let previous_arm = state.arm;
        state.arm = arm;
        update(state);
        let current = command_of(id, state);
        let reason = if current != previous {
            Some(PublishReason::Changed)
        } else if state.arm != previous_arm {
            Some(PublishReason::Refreshed)
        } else {
            None
        };
        let effects = reason
            .map(|reason| StatusEffect::Publish {
                status: status_build::to_status(&current, self.ttl_secs, now, false),
                reason,
            })
            .into_iter()
            .collect();
        let revision = self.bump_revision();
        StatusOutcome {
            effects,
            revision,
            pubkey: Some(id.to_string()),
        }
    }

    fn empty_outcome(&self, pubkey: Option<&str>) -> StatusOutcome {
        StatusOutcome {
            effects: Vec::new(),
            revision: self.revision,
            pubkey: pubkey.map(str::to_string),
        }
    }

    fn bump_revision(&mut self) -> u64 {
        self.revision = self.revision.saturating_add(1);
        self.revision
    }
}

fn command_of(pubkey: &str, state: &SessionStatus) -> StatusCommand {
    let product_state = crate::session_state::SessionState::classify(
        state.live,
        state.working,
        state.automatic_delivery,
    );
    StatusCommand {
        pubkey: pubkey.to_string(),
        channels: state.channels.iter().cloned().collect(),
        title: state.title.clone(),
        state: product_state,
        host: state.info.host.clone(),
        slug: state.info.slug.clone(),
        rel_cwd: state.info.rel_cwd.clone(),
        dispatch_event: state.info.dispatch_event.clone(),
    }
}
