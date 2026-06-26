use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::Mutex as AsyncMutex;

/// How long a channel stays verified without a re-check (5 min). Heartbeats
/// fire every 30 s; only the first one per window hits the relay.
pub(crate) const READY_TTL_SECS: u64 = 300;

/// Context for a channel readiness check.
pub struct ChannelCtx<'a> {
    /// NIP-29 group h-tag the publish targets.
    pub channel: &'a str,
    /// Pubkey (hex) that must be at least a member before the publish proceeds.
    pub expect_member: &'a str,
    /// Soft parent hint: the h-tag of the parent group to ensure first when
    /// `channel` doesn't yet exist on the relay. Ignored when the channel is
    /// already present (the relay's stored hierarchy wins). `None` creates the
    /// channel as a top-level group.
    pub parent_hint: Option<&'a str>,
}

/// Outcome of a readiness check.
#[derive(Debug)]
pub enum ChannelGate {
    /// Channel was already ready; nothing touched.
    Ready,
    /// Channel was missing, incomplete, or needed repairs; corrected.
    Repaired,
    /// Could not fully verify or repair (relay unreachable, no management key,
    /// etc.). The publish proceeds anyway (fail-open).
    Degraded,
}

struct ChannelSlot {
    verified_at: Option<Instant>,
    /// Per-channel single-flight: concurrent readiness checks coalesce on this.
    inflight: Arc<AsyncMutex<()>>,
}

/// In-process TTL'd cache tracking which channels are known-ready.
pub struct ChannelReadiness {
    inner: Mutex<HashMap<String, ChannelSlot>>,
}

impl Default for ChannelReadiness {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl ChannelReadiness {
    /// Returns `(is_ready_right_now, inflight_lock_for_this_channel)`.
    /// The caller acquires the inflight lock before doing any relay I/O, then
    /// calls this again after acquiring to double-check (another task may have
    /// repaired it while we waited for the lock).
    pub(crate) fn check(&self, channel: &str) -> (bool, Arc<AsyncMutex<()>>) {
        let mut map = self.inner.lock().expect("readiness map poisoned");
        let slot = map.entry(channel.to_string()).or_insert_with(|| ChannelSlot {
            verified_at: None,
            inflight: Arc::new(AsyncMutex::new(())),
        });
        let ready = slot
            .verified_at
            .map(|t| t.elapsed().as_secs() < READY_TTL_SECS)
            .unwrap_or(false);
        (ready, slot.inflight.clone())
    }

    pub(crate) fn mark_ready(&self, channel: &str) {
        let mut map = self.inner.lock().expect("readiness map poisoned");
        let slot = map.entry(channel.to_string()).or_insert_with(|| ChannelSlot {
            verified_at: None,
            inflight: Arc::new(AsyncMutex::new(())),
        });
        slot.verified_at = Some(Instant::now());
    }

    /// Invalidate a channel (e.g. after an observed relay-side roster change),
    /// forcing a re-verify on the next publish.
    pub fn invalidate(&self, channel: &str) {
        if let Ok(mut map) = self.inner.lock() {
            if let Some(slot) = map.get_mut(channel) {
                slot.verified_at = None;
            }
        }
    }
}
