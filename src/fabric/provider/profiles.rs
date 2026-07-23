use super::Nip29Provider;
use crate::fabric::RawEnvelope;
use nostr_sdk::prelude::{Filter, Kind, PublicKey};
use std::time::Duration;

const PROFILE_FETCH_TIMEOUT: Duration = Duration::from_secs(4);

impl Nip29Provider {
    pub(crate) async fn fetch_and_cache_profile_name(
        &self,
        pubkey: &str,
        _now: u64,
    ) -> Option<String> {
        let author = PublicKey::from_hex(pubkey).ok()?;
        let filter = Filter::new().author(author).kind(Kind::from(0u16)).limit(1);
        let event = self
            .transport
            .fetch(filter, PROFILE_FETCH_TIMEOUT)
            .await
            .ok()?
            .into_iter()
            .max_by_key(|event| event.created_at)?;
        self.with_store(|store| {
            self.materialize(&RawEnvelope::Nostr(event), store);
        });
        self.with_store(|store| {
            store
                .get_profile(pubkey)
                .ok()
                .flatten()
                .map(|profile| profile.name)
                .filter(|name| !name.is_empty())
        })
    }
}
