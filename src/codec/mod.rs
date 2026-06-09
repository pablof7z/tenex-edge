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

pub trait Codec: Send + Sync {
    /// Stable name of this wire shape (e.g. `"kind1"`).
    fn name(&self) -> &'static str;

    /// Encode a domain event into an unsigned event template. Signing (which
    /// stamps the author + created_at) happens in the transport layer.
    fn encode(&self, ev: &DomainEvent) -> anyhow::Result<EventBuilder>;

    /// Decode a signed event into a domain event, or `None` if this codec
    /// doesn't recognize it.
    fn decode(&self, event: &Event) -> Option<DomainEvent>;
}
