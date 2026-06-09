//! Transport — a thin adapter over `nostr-sdk` (M1 §2).
//!
//! Speaks wire events only: connect to relays (with NIP-42 auto-AUTH), publish
//! signed events, subscribe with filters, one-shot fetch for resolution. It
//! knows nothing of domain meaning — the codec owns that.
//!
//! (M1 names NMP as the eventual transport. NMP turned out to be a full
//! cross-platform app *kernel*, a poor fit for a headless CLI; the wire output
//! is identical standard Nostr either way, and this whole layer sits behind the
//! codec seam, so an NMP-backed transport remains a drop-in replacement.)

use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::sync::broadcast;

pub struct Transport {
    client: Client,
    pub pubkey: PublicKey,
}

impl Transport {
    /// Connect to the configured relays and authenticate.
    pub async fn connect(relays: &[String], keys: Keys) -> Result<Self> {
        let pubkey = keys.public_key();
        let opts = ClientOptions::default().automatic_authentication(true);
        let client = Client::builder().signer(keys).opts(opts).build();
        for r in relays {
            client
                .add_relay(r)
                .await
                .with_context(|| format!("adding relay {r}"))?;
        }
        client.connect().await;
        client.wait_for_connection(Duration::from_secs(8)).await;

        // Force NIP-42 AUTH to complete BEFORE any subscription. On auth-gated
        // relays a REQ opened pre-auth is closed by the relay and never delivers
        // live events; `fetch_events` carries the auth-required retry, so this
        // warm-up authenticates the connection. No-op on open relays.
        let warmup = Filter::new().kind(Kind::from(0u16)).limit(1);
        let _ = client.fetch_events(warmup, Duration::from_secs(5)).await;

        Ok(Self { client, pubkey })
    }

    /// Sign (with our key) and publish an event template.
    pub async fn publish_builder(&self, builder: EventBuilder) -> Result<EventId> {
        let out = self
            .client
            .send_event_builder(builder)
            .await
            .context("publishing event")?;
        Ok(out.val)
    }

    /// One-shot query (used for resolution — e.g. fetch a `kind:0` profile).
    pub async fn fetch(&self, filter: Filter, timeout: Duration) -> Result<Vec<Event>> {
        let events = self
            .client
            .fetch_events(filter, timeout)
            .await
            .context("fetching events")?;
        Ok(events.into_iter().collect())
    }

    /// Open long-lived subscriptions (one per filter). Incoming events arrive on
    /// [`Transport::notifications`].
    pub async fn subscribe(&self, filters: Vec<Filter>) -> Result<()> {
        for f in filters {
            self.client
                .subscribe(f, None)
                .await
                .context("subscribing")?;
        }
        Ok(())
    }

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.client.notifications()
    }

    pub async fn shutdown(&self) {
        self.client.disconnect().await;
    }
}
