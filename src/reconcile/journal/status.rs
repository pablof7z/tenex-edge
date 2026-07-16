use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::Timestamp;

/// Inputs needed to replay one committed status transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusDrive {
    SessionStarted(StatusSessionStartedArgs),
    TurnStarted {
        pubkey: String,
        at: Timestamp,
    },
    TurnEnded {
        pubkey: String,
        at: Timestamp,
    },
    TitleSet {
        pubkey: String,
        title: String,
        at: Timestamp,
    },
    ChannelsChanged {
        pubkey: String,
        channels: BTreeSet<String>,
        at: Timestamp,
    },
    Tick {
        pubkey: String,
        automatic_delivery: bool,
        at: Timestamp,
    },
    SessionEnded {
        pubkey: String,
        at: Timestamp,
    },
    /// An operator deliberately destroyed the session and requested immediate
    /// public disappearance instead of the ordinary ended-session TTL.
    SessionRevoked {
        pubkey: String,
        at: Timestamp,
    },
}

impl StatusDrive {
    pub fn at(&self) -> Timestamp {
        match self {
            Self::SessionStarted(args) => args.at,
            Self::TurnStarted { at, .. }
            | Self::TurnEnded { at, .. }
            | Self::TitleSet { at, .. }
            | Self::ChannelsChanged { at, .. }
            | Self::Tick { at, .. }
            | Self::SessionEnded { at, .. }
            | Self::SessionRevoked { at, .. } => *at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusSessionStartedArgs {
    pub pubkey: String,
    pub host: String,
    pub slug: String,
    pub rel_cwd: String,
    pub channels: BTreeSet<String>,
    pub working: bool,
    pub automatic_delivery: bool,
    pub title: String,
    #[serde(default)]
    pub dispatch_event: Option<String>,
    pub at: Timestamp,
}
