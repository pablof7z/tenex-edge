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
//! Phase 6 adds `SendIntent`, `OutboundReceipt`, and `provider.send()` for
//! canonical dual-write outbound messages alongside the legacy inbox path.

use crate::domain::{AgentRef, DomainEvent, Mention};
use crate::fabric::kind1::wire::Kind1WireCodec;
use crate::fabric::nostr_delivery::NostrDelivery;
use crate::fabric::{MaterializationOutcome, RawEnvelope, WireCodec};
use crate::state::{SessionRecord, Store};
use crate::transport::Transport;
use crate::util::now_secs;
use anyhow::Result;
use nostr_sdk::EventBuilder;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Fabric identifier used in all canonical origin rows.
pub const FABRIC: &str = "kind1-nip29";

// ── Phase 6 send types ────────────────────────────────────────────────────────

/// All inputs needed to publish one outbound message.
pub struct SendIntent {
    /// Sender's identity on the fabric.
    pub from: AgentRef,
    /// Recipient's pubkey (hex).
    pub to_pubkey: String,
    /// Project slug (NIP-29 group name).
    pub project: String,
    /// Message body.
    pub body: String,
    /// When `Some`, only the matching recipient session should surface it.
    pub target_session: Option<String>,
    /// The sender's own session id (return envelope for replies).
    pub from_session: Option<String>,
    /// Existing canonical thread id to attach to. `None` → a new thread root
    /// is created from the published event id (Phase 7 will refine).
    pub thread_id: Option<String>,
}

impl SendIntent {
    /// Convert to the `Mention` domain event used by the wire codec and the
    /// legacy local-delivery path. Both callers see the same struct.
    pub fn to_mention(&self) -> Mention {
        Mention {
            from: self.from.clone(),
            to_pubkey: self.to_pubkey.clone(),
            project: self.project.clone(),
            body: self.body.clone(),
            target_session: self.target_session.clone(),
            from_session: self.from_session.clone(),
        }
    }
}

/// Result of a successful `provider.send()`.
pub struct OutboundReceipt {
    /// Hex event id of the published Nostr event.
    pub native_event_id: String,
    /// Canonical `message_id` inserted into the read-model `messages` table.
    pub message_id: String,
    /// Sync state stored on the canonical message (`"published"` or `"accepted"`).
    pub sync_state: String,
}

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
    /// Stable hash of the sorted relay URL set. Used as the `provider_instance`
    /// column in canonical origin rows, making them deterministic across daemon
    /// restarts. Derived once at construction from `cfg.relays`.
    pub provider_instance: String,
}

impl Kind1Nip29Provider {
    pub fn new(
        transport: Arc<Transport>,
        store: Arc<Mutex<Store>>,
        user_nsec: Option<String>,
        relays: &[String],
    ) -> Self {
        let delivery = NostrDelivery::new(transport.clone());
        let wire = Kind1WireCodec;
        let provider_instance = derive_provider_instance(relays);
        Self {
            delivery,
            wire,
            store,
            transport,
            user_nsec,
            provider_instance,
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

    // ── send ──────────────────────────────────────────────────────────────────

    /// Publish one outbound message and dual-write canonical read-model rows.
    ///
    /// Behavior:
    /// 1. Encode the intent as a Mention DomainEvent and publish it.
    ///    On publish error the error propagates unchanged (same as today).
    /// 2. On success, lock the store ONCE and write:
    ///    - `projects` / `project_origins` (idempotent)
    ///    - `threads` / `thread_origins` (idempotent, keyed by event id)
    ///    - `messages` with `sync_state="published"` (idempotent on native_event_id)
    ///    - `message_recipients`
    /// 3. Return `OutboundReceipt` carrying the published event id and the new
    ///    canonical `message_id`.
    ///
    /// The legacy inbox path is NOT touched here — local delivery is the
    /// caller's responsibility (rpc_send_message keeps route_mention_into_with_id).
    pub async fn send(
        &self,
        intent: SendIntent,
        agent_keys: &nostr_sdk::Keys,
    ) -> Result<OutboundReceipt> {
        // Build the wire event from the intent's Mention.
        let mention = intent.to_mention();
        let builder = self.wire.encode(&DomainEvent::Mention(mention))?;

        // Publish. On error, propagate immediately — no canonical row written.
        let event_id = self
            .transport
            .publish_signed(builder, agent_keys)
            .await?;
        let eid_hex = event_id.to_hex();

        // Dual-write canonical rows (single lock, no await inside).
        let now = now_secs();
        let pi = self.provider_instance.clone();
        let (message_id,) = self.with_store(|s| -> Result<(String,)> {
            let project_id = s.ensure_project_origin(
                FABRIC,
                &pi,
                &intent.project,
                &intent.project,
                now,
            )?;
            let thread_id = if let Some(tid) = intent.thread_id.as_deref() {
                tid.to_string()
            } else {
                // Each outbound root is its own thread for now; Phase 7 refines.
                s.ensure_thread_origin(&project_id, FABRIC, &pi, &eid_hex, now)?
            };
            let message_id = s.record_message(
                &thread_id,
                &intent.from.pubkey,
                &intent.body,
                now,
                "outbound",
                "published",
                Some(&eid_hex),
            )?;
            s.add_message_recipient(
                &message_id,
                &intent.to_pubkey,
                intent.target_session.as_deref(),
            )?;
            Ok((message_id,))
        })?;

        Ok(OutboundReceipt {
            native_event_id: eid_hex,
            message_id,
            sync_state: "published".into(),
        })
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
            let pi = self.provider_instance.clone();
            for ev in events {
                let env = RawEnvelope::Nostr(ev);
                let outcome = self.with_store(|s| {
                    crate::fabric::materialize(&env, &hosted, owners, now, &pi, s)
                });
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
        crate::fabric::materialize(env, hosted, owners, now, &self.provider_instance, store)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn with_store<R>(&self, f: impl FnOnce(&Store) -> R) -> R {
        let g = self.store.lock().expect("store mutex poisoned");
        f(&g)
    }
}

// ── module-level helpers ──────────────────────────────────────────────────────

/// Derive a stable `provider_instance` string from the relay URL set.
///
/// Sorts + deduplicates the relay URLs, joins them with `|`, hashes with
/// `DefaultHasher` (fixed seed = 0 at the point the hasher is reset), and
/// formats the result as 16 hex digits.  The value is deterministic for the
/// same relay set across daemon restarts on the same machine.
fn derive_provider_instance(relays: &[String]) -> String {
    let mut sorted: Vec<&str> = relays.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let joined = sorted.join("|");
    let mut h = DefaultHasher::new();
    joined.hash(&mut h);
    format!("{:016x}", h.finish())
}
