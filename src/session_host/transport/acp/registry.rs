//! Process-global ACP child registry and exit reaper.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::mpsc;

use super::super::acp_runtime::AcpRuntime;
use crate::rpc_harness::{RpcHandle, SessionUpdate};

/// A live ACP/app-server child plus its native session token.
pub(super) struct AcpChild {
    pub(super) handle: RpcHandle,
    /// ACP `sessionId` or app-server `threadId`.
    pub(super) native_id: String,
    pub(super) cwd: PathBuf,
    /// Captured transcript + running-turn state, fed by the update-drain task.
    pub(super) runtime: Arc<Mutex<AcpRuntime>>,
}

pub(super) fn registry() -> &'static Mutex<HashMap<String, AcpChild>> {
    static REG: OnceLock<Mutex<HashMap<String, AcpChild>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a freshly-opened child, drain its updates into shared runtime state,
/// and reap both the registry entry and zombie when it exits.
pub(super) fn register_child(
    endpoint_id: &str,
    handle: RpcHandle,
    native_id: String,
    cwd: PathBuf,
    mut updates: mpsc::UnboundedReceiver<SessionUpdate>,
) {
    let runtime = Arc::new(Mutex::new(AcpRuntime::default()));
    let rt_updates = runtime.clone();
    tokio::spawn(async move {
        while let Some(update) = updates.recv().await {
            if let Ok(mut runtime) = rt_updates.lock() {
                runtime.note_update(&update.method, &update.params);
            }
        }
    });

    let reaper_handle = handle.clone();
    let reaper_id = endpoint_id.to_string();
    tokio::spawn(async move {
        reaper_handle.wait_exit().await;
        registry().lock().unwrap().remove(&reaper_id);
        tracing::debug!(endpoint = %reaper_id, "acp child exited; registry entry reaped");
    });
    registry().lock().unwrap().insert(
        endpoint_id.to_string(),
        AcpChild {
            handle,
            native_id,
            cwd,
            runtime,
        },
    );
}
