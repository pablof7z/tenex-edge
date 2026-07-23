use serde_json::Value;

/// Return why a hook should fail open before agent/PID discovery or RPC work.
pub(super) fn inapplicable() -> Option<(&'static str, Value)> {
    // Socket absence is authoritative for the fast no-spawn path. A stale
    // socket still falls through to the bounded RPC attempt.
    (!crate::daemon::socket_path().exists()).then(|| ("daemon-unavailable", serde_json::json!({})))
}
