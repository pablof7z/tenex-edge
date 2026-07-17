use super::{session_claims::SessionClaim, Channel, Profile, Session, Status, Store};
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

    pub(crate) fn session_identity(
        self,
        pubkey: &str,
    ) -> Result<Option<crate::identity::SessionIdentity>> {
        self.store.session_identity(pubkey)
    }

    pub(crate) fn get_session(self, pubkey: &str) -> Result<Option<Session>> {
        self.store.get_session(pubkey)
    }

    pub(crate) fn has_live_delivery_path(self, session: &Session) -> bool {
        crate::session_host::session_has_live_delivery_path(self.store, session)
    }

    pub(crate) fn list_active_session_claims(self, now: u64) -> Result<Vec<SessionClaim>> {
        self.store.list_active_session_claims(now)
    }

    pub(crate) fn get_status(self, pubkey: &str, channel_h: &str) -> Result<Option<Status>> {
        self.store.get_status(pubkey, channel_h)
    }
}
