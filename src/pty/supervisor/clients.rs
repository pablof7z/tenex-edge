use super::{trace, trace_bytes};
use crate::pty::{ConditionalKillOutcome, PresentationSnapshot};
use anyhow::Result;
use std::collections::VecDeque;
use std::io::{BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

type Client = Arc<Mutex<UnixStream>>;
type ChangeCallback = Arc<dyn Fn(PresentationSnapshot) + Send + Sync>;

pub(super) struct ClientSet {
    entries: Vec<Client>,
    attachment_epoch: u64,
    changed_at: u64,
    on_change: ChangeCallback,
}

pub(super) type Clients = Arc<Mutex<ClientSet>>;

pub(super) fn new(pty_id: String) -> Clients {
    new_with_callback(Arc::new(move |presentation| {
        notify_daemon(pty_id.clone(), presentation);
    }))
}

pub(super) fn snapshot(clients: &Clients) -> PresentationSnapshot {
    snapshot_locked(&clients.lock().unwrap())
}

pub(super) fn kill_if_headless(
    clients: &Clients,
    expected_epoch: u64,
    terminate_confirmed: impl FnOnce(PresentationSnapshot) -> Result<()>,
) -> Result<ConditionalKillOutcome> {
    let state = clients.lock().unwrap();
    let presentation = snapshot_locked(&state);
    if presentation.is_headless() && presentation.attachment_epoch == expected_epoch {
        // Hold the attachment lock through the kill request. A concurrent ATTACH
        // cannot register between the predicate and termination.
        terminate_confirmed(presentation)?;
        return Ok(ConditionalKillOutcome::Killed { presentation });
    }
    Ok(ConditionalKillOutcome::PresentationChanged { presentation })
}

pub(super) fn attach_client(
    mut reader: BufReader<UnixStream>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    clients: &Clients,
    backlog: &Arc<Mutex<VecDeque<u8>>>,
) -> Result<()> {
    let mut output = reader.get_ref().try_clone()?;
    let remembered = backlog.lock().unwrap().iter().copied().collect::<Vec<_>>();
    if !remembered.is_empty() {
        let _ = output.write_all(&remembered);
    }
    let client = Arc::new(Mutex::new(output));
    let (on_change, presentation) = {
        let mut state = clients.lock().unwrap();
        state.entries.push(client.clone());
        advance_epoch(&mut state, 1);
        (state.on_change.clone(), snapshot_locked(&state))
    };
    on_change(presentation);
    let clients = clients.clone();
    std::thread::spawn(move || {
        let mut buf = [0_u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    trace("supervisor attach eof");
                    break;
                }
                Ok(n) => {
                    trace_bytes("supervisor attach", &buf[..n]);
                    let mut writer = writer.lock().unwrap();
                    let result = writer.write_all(&buf[..n]).and_then(|_| writer.flush());
                    if result.is_err() {
                        trace("supervisor attach write error");
                        break;
                    }
                }
                Err(_) => {
                    trace("supervisor attach read error");
                    break;
                }
            }
        }
        let changed = {
            let mut state = clients.lock().unwrap();
            let before = state.entries.len();
            state.entries.retain(|entry| !Arc::ptr_eq(entry, &client));
            let removed = before.saturating_sub(state.entries.len());
            advance_epoch(&mut state, removed);
            (removed > 0).then(|| (state.on_change.clone(), snapshot_locked(&state)))
        };
        if let Some((on_change, presentation)) = changed {
            on_change(presentation);
        }
    });
    Ok(())
}

pub(super) fn fanout(clients: &Clients, bytes: &[u8]) {
    let changed = {
        let mut state = clients.lock().unwrap();
        let before = state.entries.len();
        state.entries.retain(|client| {
            let Ok(mut stream) = client.lock() else {
                return false;
            };
            stream.write_all(bytes).and_then(|_| stream.flush()).is_ok()
        });
        let removed = before.saturating_sub(state.entries.len());
        advance_epoch(&mut state, removed);
        (removed > 0).then(|| (state.on_change.clone(), snapshot_locked(&state)))
    };
    if let Some((on_change, presentation)) = changed {
        on_change(presentation);
    }
}

fn snapshot_locked(state: &ClientSet) -> PresentationSnapshot {
    PresentationSnapshot {
        attached_clients: u64::try_from(state.entries.len()).expect("PTY client count overflow"),
        attachment_epoch: state.attachment_epoch,
        changed_at: state.changed_at,
    }
}

fn advance_epoch(state: &mut ClientSet, transitions: usize) {
    let transitions = u64::try_from(transitions).expect("PTY attachment transition overflow");
    state.attachment_epoch = state
        .attachment_epoch
        .checked_add(transitions)
        .expect("PTY attachment epoch exhausted");
    state.changed_at = crate::util::now_secs();
}

fn new_with_callback(on_change: ChangeCallback) -> Clients {
    Arc::new(Mutex::new(ClientSet {
        entries: Vec::new(),
        attachment_epoch: 0,
        changed_at: crate::util::now_secs(),
        on_change,
    }))
}

fn notify_daemon(pty_id: String, presentation: PresentationSnapshot) {
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(async move {
            let Ok(mut client) = crate::daemon::client::Client::connect_running().await else {
                return;
            };
            client
                .call(
                    "pty_presentation_changed",
                    serde_json::json!({
                        "pty_id": pty_id,
                        "presentation": presentation,
                    }),
                )
                .await
                .ok();
        });
    });
}

#[cfg(test)]
#[path = "clients/tests.rs"]
mod tests;
