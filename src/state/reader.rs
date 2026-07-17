use super::{AgentAvailability, Channel, Profile, Session, SessionStanding, Status, Store};
use anyhow::Result;

/// Read-only capability for callers that assemble views from the store.
#[derive(Clone, Copy)]
pub(crate) struct StoreReader<'a> {
    store: &'a Store,
}

impl Store {
    pub(crate) fn reader(&self) -> StoreReader<'_> {
        StoreReader { store: self }
    }
}

impl StoreReader<'_> {
    pub(crate) fn get_channel(self, channel_h: &str) -> Result<Option<Channel>> {
        self.store.get_channel(channel_h)
    }

    pub(crate) fn list_channels(self) -> Result<Vec<Channel>> {
        self.store.list_channels()
    }

    pub(crate) fn channel_parent(self, channel_h: &str) -> Result<Option<String>> {
        self.store.channel_parent(channel_h)
    }

    pub(crate) fn root_channel_of(self, channel_h: &str) -> Result<Option<String>> {
        self.store.root_channel_of(channel_h)
    }

    pub(crate) fn is_root_channel(self, channel_h: &str) -> Result<bool> {
        self.store.is_root_channel(channel_h)
    }

    pub(crate) fn get_profile(self, pubkey: &str) -> Result<Option<Profile>> {
        self.store.get_profile(pubkey)
    }

    pub(crate) fn resolve_slug_for_pubkey(self, pubkey: &str) -> Result<Option<String>> {
        self.store.resolve_slug_for_pubkey(pubkey)
    }

    pub(crate) fn list_local_session_pubkeys(self) -> Result<Vec<String>> {
        self.store.list_local_session_pubkeys()
    }

    pub(crate) fn list_agent_roster_for_channel(
        self,
        channel_h: &str,
    ) -> Result<Vec<AgentAvailability>> {
        self.store.list_agent_roster_for_channel(channel_h)
    }

    pub(crate) fn list_agent_roster(self) -> Result<Vec<AgentAvailability>> {
        self.store.list_agent_roster()
    }

    pub(crate) fn session_identity(
        self,
        pubkey: &str,
    ) -> Result<Option<crate::identity::SessionIdentity>> {
        self.store.session_identity(pubkey)
    }

    pub(crate) fn list_running_sessions(self) -> Result<Vec<Session>> {
        self.store.list_running_sessions()
    }

    pub(crate) fn get_session(self, pubkey: &str) -> Result<Option<Session>> {
        self.store.get_session(pubkey)
    }

    pub(crate) fn list_retained_session_standing(self, now: u64) -> Result<Vec<SessionStanding>> {
        self.store.list_retained_session_standing(now)
    }

    pub(crate) fn has_live_delivery_path(self, session: &Session) -> bool {
        crate::session_host::session_has_live_delivery_path(self.store, session)
    }

    pub(crate) fn get_status(self, pubkey: &str, channel_h: &str) -> Result<Option<Status>> {
        self.store.get_status(pubkey, channel_h)
    }

    pub(crate) fn live_status_for_channel(self, channel_h: &str, now: u64) -> Result<Vec<Status>> {
        self.store.live_status_for_channel(channel_h, now)
    }
}
