//! Narrow direct-wire adapter over `nostr-sdk`.
//!
//! Speaks the narrow direct-wire operations NMP cannot currently express:
//! one-shot fetches, the profile indexer copy, and the connectivity probe. NMP
//! owns group publication, signing, receipts, retries, and live acquisition.

use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::time::Duration;

pub(crate) struct Transport {
    client: Client,
    /// URLs of the main NIP-29 relay(s) — the explicit broadcast target for
    /// every publish. `nostr-relay-pool` gates BOTH `send_event`'s implicit
    /// "all WRITE-flagged relays" broadcast AND an explicitly-targeted
    /// `send_event_to` by the SAME per-relay `WRITE` flag (there is no flag
    /// combination meaning "writable only when explicitly addressed, excluded
    /// from broadcast"). Since the indexer relay must carry `WRITE` (so the
    /// explicit kind:0 publish below can reach it at all), publishing here
    /// MUST target `write_relay_urls` explicitly rather than call the
    /// pool's implicit broadcast — otherwise every publish would fan out to
    /// the indexer too and it would reject non-kind:0 events.
    write_relay_urls: Vec<String>,
    /// URL of the profile indexer relay, if configured. kind:0 profiles are
    /// routed here explicitly via [`Transport::publish_profile_event`]; it is added
    /// with full READ+WRITE flags (needed for that explicit publish to
    /// succeed) but is deliberately excluded from `write_relay_urls`, so it
    /// never receives the broadcast publishes above and therefore never
    /// rejects NIP-29 kinds ("blocked: kind 9000 is not allowed") — which
    /// would otherwise pollute `assert_relay_accepted`'s joined-reason verdict.
    indexer_url: Option<String>,
}

// ── relay-ack assertion ─────────────────────────────────────────────────────

/// Fail unless at least one relay accepted the publish. `nostr-sdk`'s
/// `send_event*` resolves `Ok` as long as the message was transmitted; the
/// actual NIP-01 `["OK", id, true|false, reason]` verdict per relay lives in
/// `output.success` / `output.failed`. An empty `success` set usually means every
/// relay rejected the event (or the connection timed out before any OK arrived),
/// so a caller reporting "published" off the bare `Ok` would be lying. A
/// "duplicate" failure is the idempotent exception: the relay already has the
/// signed event, so durability is satisfied.
fn assert_relay_accepted(output: &Output<EventId>, event: Option<&Event>) -> Result<()> {
    if !output.success.is_empty() {
        return Ok(());
    }
    if output
        .failed
        .values()
        .any(|r| r.to_ascii_lowercase().contains("duplicate"))
    {
        return Ok(());
    }
    let reasons: Vec<String> = output
        .failed
        .values()
        .filter(|r| !r.is_empty())
        .cloned()
        .collect();
    if reasons.is_empty() {
        crate::relay_log::log_relay_rejection("no relay returned OK (timeout)", event);
        anyhow::bail!("no relay accepted the event (timeout or no OK received)");
    }
    let msg = reasons.join("; ");
    crate::relay_log::log_relay_rejection(&msg, event);
    anyhow::bail!("relay rejected event: {msg}");
}

// ── Transport ─────────────────────────────────────────────────────────────────

/// `connect_with_indexer` kicks off the relay connection in the background and
/// returns immediately (see its doc), and the daemon opens its RPC accept loop
/// before that connection finishes (relay warmup runs off the startup critical
/// path too) — so a client that races a freshly-spawned daemon can reach a
/// publish call before any relay has finished its handshake. `nostr-relay-pool`
/// treats that as a hard per-relay failure (`Error::NotConnected`) rather than
/// queuing, so every publish path below waits (briefly, bounded) for the
/// connection first. Once connected this returns immediately, so it's free on
/// the steady-state path.
const PUBLISH_CONNECT_WAIT: Duration = Duration::from_secs(8);

impl Transport {
    /// Connect to the configured main relays plus an optional profile indexer
    /// relay. The indexer is added with full READ+WRITE flags (both are needed:
    /// READ for kind:0 lookups, WRITE for the explicit kind:0 publish), but
    /// every broadcast publish below explicitly targets `write_relay_urls`
    /// (the main relays only), so the indexer never receives — and therefore
    /// never rejects — NIP-29 group events.
    pub(crate) async fn connect_with_indexer(
        relays: &[String],
        indexer_url: Option<&str>,
        keys: Keys,
    ) -> Result<Self> {
        let opts = ClientOptions::default().automatic_authentication(true);
        let client = Client::builder().signer(keys).opts(opts).build();
        for r in relays {
            client
                .add_relay(r)
                .await
                .with_context(|| format!("adding relay {r}"))?;
        }
        // Full default flags (READ+WRITE+PING): READ so kind:0 lookups still
        // resolve here, WRITE so the explicit profile copy below can reach
        // it — `send_event_to` enforces the per-relay WRITE flag even when the
        // caller names the relay explicitly, so a READ-only add would make
        // every indexer publish fail with `write actions are disabled`.
        if let Some(url) = indexer_url {
            if !url.is_empty() {
                client
                    .add_relay(url)
                    .await
                    .with_context(|| format!("adding indexer relay {url}"))?;
            }
        }
        // Initiate the connection in the BACKGROUND (not awaited here): the daemon
        // builds its Transport before the accept loop is spawned, and awaiting
        // `connect()` — which can block on the relay handshake under load — would
        // stall even store-only RPCs. `warmup()` awaits real connectivity + AUTH.
        let cc = client.clone();
        tokio::spawn(async move { cc.connect().await });
        Ok(Self {
            client,
            write_relay_urls: relays.to_vec(),
            indexer_url: indexer_url.filter(|s| !s.is_empty()).map(String::from),
        })
    }

    /// Block (bounded) until the direct provider connection is established and
    /// complete any relay-requested AUTH before startup publishes or one-shot
    /// fetches. NMP's live observations use their own public access context.
    pub(crate) async fn warmup(&self) {
        self.client
            .wait_for_connection(Duration::from_secs(8))
            .await;
        let warmup = Filter::new().kind(Kind::from(0u16)).limit(1);
        let _ = self
            .client
            .fetch_events(warmup, Duration::from_secs(5))
            .await;
    }

    /// Publish the doctor's uniquely-tagged probe to the configured app relays.
    pub(crate) async fn publish_probe_checked(&self, marker: &str) -> Result<EventId> {
        let builder = EventBuilder::new(Kind::from(1u16), format!("mosaico doctor {marker}"))
            .tags([Tag::parse(["h", marker])?]);
        self.client.wait_for_connection(PUBLISH_CONNECT_WAIT).await;
        let out = self
            .client
            .send_event_builder_to(self.write_relay_urls.iter().cloned(), builder)
            .await
            .context("publishing event")?;
        assert_relay_accepted(&out, None)?;
        Ok(out.val)
    }

    /// Copy a kind:0 profile to BOTH the indexer and the main relay(s). NMP's
    /// facade deliberately exposes no arbitrary pinned-host write, while
    /// author-outbox routing requires NIP-65 evidence that profiles may not yet
    /// have. This is the sole product write outside NMP.
    ///
    /// The main relay accepts kind:0, so profiles go to BOTH the
    /// indexer relay AND the main NIP-29 relay(s) — the main relay accepts
    /// kind:0 fine, so profiles must land there too (agent/backend name
    /// resolution shouldn't depend on the indexer alone). Falls back to the
    /// main relays when no indexer is configured.
    pub(crate) async fn publish_profile_event(&self, signed: &Event) -> Result<EventId> {
        crate::relay_log::log_outgoing_event(signed);
        self.client.wait_for_connection(PUBLISH_CONNECT_WAIT).await;
        let mut urls = self.write_relay_urls.clone();
        if let Some(indexer) = &self.indexer_url {
            if !urls.contains(indexer) {
                urls.push(indexer.clone());
            }
        }
        if urls.is_empty() {
            anyhow::bail!("cannot publish a profile without a configured relay");
        }
        let out = self
            .client
            .send_event_to(urls, signed)
            .await
            .context("publishing signed event to target relays")?;
        assert_relay_accepted(&out, Some(signed))?;
        Ok(out.val)
    }

    /// One-shot query (used for resolution — e.g. fetch a `kind:0` profile).
    pub(crate) async fn fetch(&self, filter: Filter, timeout: Duration) -> Result<Vec<Event>> {
        let events = self
            .client
            .fetch_events(filter, timeout)
            .await
            .context("fetching events")?;
        Ok(events.into_iter().collect())
    }

    pub(crate) async fn shutdown(&self) {
        self.client.disconnect().await;
    }
}

#[cfg(test)]
mod tests;
