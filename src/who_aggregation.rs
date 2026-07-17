//! Canonical store aggregation for both `who` renderers.
//!
//! CLI JSON/text and the agent XML tree project the same captured channel,
//! session, capability, and status facts. Renderer-specific layout stays out
//! of this module.

use crate::state::{AgentAvailability, Channel, Session, Status, Store};
use anyhow::{Context, Result};
use std::collections::BTreeMap;

pub(crate) struct WhoAggregation {
    pub(crate) channels: Vec<Channel>,
    pub(crate) local_sessions: Vec<Session>,
    pub(crate) agents: Vec<AgentAvailability>,
    statuses: BTreeMap<String, Vec<Status>>,
}

impl WhoAggregation {
    pub(crate) fn load(store: &Store, now: u64) -> Result<Self> {
        let channels = store
            .list_channels()
            .context("who aggregation: failed to list channels")?;
        let local_sessions = store
            .list_alive_sessions()
            .context("who aggregation: failed to list live local sessions")?;
        let agents = store
            .list_agent_roster()
            .context("who aggregation: failed to list agent capabilities")?;
        let mut statuses = BTreeMap::<String, Vec<Status>>::new();
        for status in store
            .list_status_sessions(None, None)
            .context("who aggregation: failed to read statuses")?
            .into_iter()
            .filter(|status| status.expiration >= now)
        {
            statuses
                .entry(status.channel_h.clone())
                .or_default()
                .push(status);
        }
        for rows in statuses.values_mut() {
            rows.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        }
        Ok(Self {
            channels,
            local_sessions,
            agents,
            statuses,
        })
    }

    pub(crate) fn statuses_for(&self, channel_h: &str) -> &[Status] {
        self.statuses
            .get(channel_h)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) fn agents_for_root(&self, root: Option<&str>) -> Vec<AgentAvailability> {
        self.agents
            .iter()
            .filter(|agent| root.is_none_or(|root| agent.channel_h == root))
            .cloned()
            .collect()
    }
}
