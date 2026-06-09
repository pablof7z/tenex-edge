//! Phase 5: `Kind1Nip29Provider` — concrete provider wrapping delivery, wire
//! codec, materializer, and lifecycle in one place.
//!
//! Design constraints (see docs/fabric-architecture.md §Phase 5):
//! - CONCRETE struct with INHERENT async methods; no async_trait, no dyn.
//! - Single-writer invariant: the provider holds a CLONE of the same
//!   `Arc<Mutex<Store>>` that `DaemonState` owns — one SQLite connection total.
//! - Dynamic per-call data (the hosted "me" set, owners, now) is passed to
//!   methods, not stored; the provider owns only stable construction-time data.
//!
//! Phase 6 will add `send` / `set_status`; stubs are documented in
//! `FabricProvider` trait (shell, not wired to `Kind1Nip29Provider`).

use crate::domain::DomainEvent;
use crate::fabric::kind1::wire::Kind1WireCodec;
use crate::fabric::nostr_delivery::NostrDelivery;
use crate::fabric::{MaterializationOutcome, RawEnvelope, WireCodec};
use crate::state::{SessionRecord, Store};
use crate::transport::Transport;
use crate::util::now_secs;
use anyhow::Result;
use nostr_sdk::EventBuilder;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ── Trait shell (documentation only; daemon calls concrete inherent methods) ───

/// Shell trait documenting the provider API surface.
///
/// Phase 5 implements the concrete methods directly on `Kind1Nip29Provider`
/// (inherent) rather than via `impl FabricProvider`, to avoid async-fn-in-trait
/// machinery. `send` and `set_status` are Phase 6; stubs listed here only.
#[allow(dead_code)]
pub trait FabricProvider {
    fn name(&self) -> &'static str;
    // async fn open_project(&self, project: &str, agent_pubkey: &str);
    // async fn send(&self, intent: SendIntent) -> Result<OutboundReceipt>;       // Phase 6
    // async fn set_status(&self, intent: StatusIntent) -> Result<OutboundReceipt>; // Phase 6
    // async fn subscribe_project(&self, scope: crate::fabric::Scope) -> Result<()>;
    // async fn catch_up_mentions(&self, rec: &SessionRecord) -> Result<usize>;
    // fn materialize(&self, env: RawEnvelope, hosted: &[String], owners: &[String], now: u64, store: &Store) -> MaterializationOutcome;
}

// ── Kind1Nip29Provider ────────────────────────────────────────────────────────

/// The first concrete provider: NIP-29 groups over Nostr kind:1 wire encoding.
///
/// Fields held at construction time (stable config). Per-call dynamic data
/// (hosted "me" set, owners, now) is received as method parameters.
pub struct Kind1Nip29Provider {
    pub delivery: NostrDelivery,
    pub wire: Kind1WireCodec,
    /// Shared store Arc — same handle as `DaemonState.store`. No new Connection.
    pub store: Arc<Mutex<Store>>,
    /// Same Arc as `DaemonState.transport` — used for lifecycle publishes only.
    pub transport: Arc<Transport>,
    /// Operator nsec for NIP-29 group management. Optional: if unset, group
    /// management is skipped and sessions still start (best-effort).
    pub user_nsec: Option<String>,
    // NOTE: `owners` and `host` are intentionally OMITTED here for Phase 5 —
    // they are passed as method arguments where needed, avoiding dead-field
    // warnings. Phase 6 will add them when send/set_status need them.
}

impl Kind1Nip29Provider {
    pub fn new(
        transport: Arc<Transport>,
        store: Arc<Mutex<Store>>,
        user_nsec: Option<String>,
    ) -> Self {
        let delivery = NostrDelivery::new(transport.clone());
        let wire = Kind1WireCodec;
        Self {
            delivery,
            wire,
            store,
            transport,
            user_nsec,
        }
    }

    // ── name ──────────────────────────────────────────────────────────────────

    pub fn name(&self) -> &'static str {
        "kind1-nip29"
    }

    // ── encode / decode ───────────────────────────────────────────────────────

    /// Encode a domain event to an `EventBuilder` via the Kind1 wire codec.
    pub fn encode(&self, ev: &DomainEvent) -> Result<EventBuilder> {
        self.wire.encode(ev)
    }

    /// Decode a raw envelope to a domain event via the Kind1 wire codec.
    pub fn decode(&self, env: &RawEnvelope) -> Option<DomainEvent> {
        self.wire.decode(env)
    }

    // ── open_project ─────────────────────────────────────────────────────────

    /// Ensure the operator owns a closed NIP-29 group for `project` and that
    /// `agent_pubkey` is a member. Best-effort: never blocks session start.
    ///
    /// This is the EXACT body of the former `ensure_group_and_membership` free
    /// function (server.rs ~466-551), minus the trailing `ensure_subscription`
    /// call which remains at the call site (rpc_session_start / reconcile_sessions)
    /// to preserve the existing double-subscribe behavior.
    pub async fn open_project(&self, project: &str, agent_pubkey: &str) {
        use nostr_sdk::prelude::Keys;
        let nsec = match &self.user_nsec {
            Some(n) => n.clone(),
            None => {
                if std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                    eprintln!(
                        "[daemon] userNsec unset; skipping NIP-29 group management for {project}"
                    );
                }
                return;
            }
        };
        let user_keys = match Keys::parse(&nsec) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("[daemon] userNsec parse failed; skipping group management: {e}");
                return;
            }
        };

        // Publish a group-management event, returning whether the relay accepted it.
        let publish = |builder, label: &'static str| {
            let transport = self.transport.clone();
            let keys = user_keys.clone();
            async move {
                match transport.publish_signed_checked(builder, &keys).await {
                    Ok(()) => true,
                    Err(e) => {
                        let benign = {
                            let s = e.to_string();
                            s.contains("already exists") || s.contains("duplicate")
                        };
                        if !benign && std::env::var("TENEX_EDGE_DEBUG").is_ok() {
                            eprintln!(
                                "[daemon] NIP-29 {label} publish failed (will retry next session): {e:#}"
                            );
                        }
                        benign
                    }
                }
            }
        };

        // 1. Create + lock the group the first time we touch this project.
        if !self.with_store(|s| s.is_group_owned(project).unwrap_or(false)) {
            let created = match crate::codec::kind1::group_create(project) {
                Ok(b) => publish(b, "9007 create-group").await,
                Err(_) => false,
            };
            let locked = if created {
                match crate::codec::kind1::group_lock_closed(project) {
                    Ok(b) => publish(b, "9002 lock-closed").await,
                    Err(_) => false,
                }
            } else {
                false
            };
            if created && locked {
                self.with_store(|s| {
                    s.mark_group_owned(project, now_secs()).ok();
                });
            }
        }

        // 2. Add this agent as a member if it isn't one already.
        if !self.with_store(|s| s.is_group_member(project, agent_pubkey).unwrap_or(false)) {
            let added = match crate::codec::kind1::group_put_user(project, agent_pubkey) {
                Ok(b) => publish(b, "9000 put-user").await,
                Err(_) => false,
            };
            if added {
                self.with_store(|s| {
                    s.upsert_group_member(project, agent_pubkey, "member", now_secs())
                        .ok();
                });
            }
        }
    }

    // ── subscribe ─────────────────────────────────────────────────────────────

    /// Forward a `Scope` subscription to the underlying delivery layer.
    /// Called by `ensure_subscription` / `resubscribe` in the daemon.
    pub async fn subscribe(&self, scope: crate::fabric::Scope) -> Result<()> {
        self.delivery.subscribe(scope).await
    }

    // ── catch_up_mentions ─────────────────────────────────────────────────────

    /// Fetch kind:1 events p-tagged to `rec.agent_pubkey` from the relay and
    /// materialize each through `crate::fabric::materialize`. Returns the number
    /// of events that triggered a mention wake.
    ///
    /// IMPORTANT: uses a single-element `hosted = [rec.agent_pubkey]` slice —
    /// intentionally NOT the daemon's full hosted set — so the Mention guard
    /// (`m.to_pubkey == me`) works exactly (relay filter already restricts).
    /// The caller fires `mention_notify.notify_waiters()` if the count > 0.
    pub async fn catch_up_mentions(
        &self,
        rec: &SessionRecord,
        owners: &[String],
    ) -> Result<usize> {
        use nostr_sdk::prelude::{Filter, Kind, PublicKey};
        let me = rec.agent_pubkey.clone();
        let pk = PublicKey::from_hex(&me)?;
        let filter = Filter::new().kind(Kind::from(1u16)).pubkey(pk).limit(50);
        let mut wake_count = 0usize;
        if let Ok(events) = self
            .transport
            .fetch(filter, Duration::from_secs(3))
            .await
        {
            let hosted = vec![me.clone()];
            let now = now_secs();
            for ev in events {
                let env = RawEnvelope::Nostr(ev);
                let outcome =
                    self.with_store(|s| crate::fabric::materialize(&env, &hosted, owners, now, s));
                // NOTE: do NOT send outcome.tail here — fetch is startup catchup only;
                // historical mentions must not be replayed onto the tail channel.
                if outcome.wake_mentions {
                    wake_count += 1;
                }
            }
        }
        Ok(wake_count)
    }

    // ── materialize ───────────────────────────────────────────────────────────

    /// Decode one raw envelope and apply all store side-effects.
    /// Delegates to `crate::fabric::materialize`.
    ///
    /// NOTE: the `store` arg is passed IN by the daemon (already locked via
    /// `state.with_store`) — this method must NOT lock `self.store` again.
    pub fn materialize(
        &self,
        env: &RawEnvelope,
        hosted: &[String],
        owners: &[String],
        now: u64,
        store: &Store,
    ) -> MaterializationOutcome {
        crate::fabric::materialize(env, hosted, owners, now, store)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn with_store<R>(&self, f: impl FnOnce(&Store) -> R) -> R {
        let g = self.store.lock().expect("store mutex poisoned");
        f(&g)
    }
}
