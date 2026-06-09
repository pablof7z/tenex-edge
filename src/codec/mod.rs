//! The codec seam (M1 §3).
//!
//! A `Codec` maps every `DomainEvent` to and from a wire envelope, and owns the
//! subscription filters that fetch them. The domain layer never names a kind or
//! a tag — all of that lives here. Swapping transports later (NIP-29, Marmot)
//! means adding another `Codec` impl; nothing in `domain` changes.

use crate::domain::DomainEvent;
use nostr_sdk::prelude::*;

pub mod kind1;

pub use kind1::Kind1Codec;

/// What to subscribe to. The runtime fills this in; the codec turns it into
/// concrete relay filters for whatever wire shape it speaks.
#[derive(Debug, Clone, Default)]
pub struct SubScope {
    /// Trusted authors (hex pubkeys). Empty = any author.
    pub authors: Vec<String>,
    /// Restrict to a single project slug, or `None` for all projects.
    pub project: Option<String>,
    /// My pubkey (hex); when set, also catch mentions addressed to me.
    pub mentions_to: Option<String>,
    /// Owner pubkey(s) (hex); when set, also discover any `kind:0` that p-tags
    /// them — i.e. agents (possibly unknown) claiming this human as owner.
    pub owners: Vec<String>,
}

pub trait Codec: Send + Sync {
    /// Stable name of this wire shape (e.g. `"kind1"`).
    fn name(&self) -> &'static str;

    /// Encode a domain event into an unsigned event template. Signing (which
    /// stamps the author + created_at) happens in the transport layer.
    fn encode(&self, ev: &DomainEvent) -> anyhow::Result<EventBuilder>;

    /// Decode a signed event into a domain event, or `None` if this codec
    /// doesn't recognize it.
    fn decode(&self, event: &Event) -> Option<DomainEvent>;

    /// Subscription filters covering every event type this codec understands.
    fn filters(&self, scope: &SubScope) -> Vec<Filter>;
}
