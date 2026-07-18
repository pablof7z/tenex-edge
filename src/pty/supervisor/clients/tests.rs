use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

fn attach(clients: &Clients) -> UnixStream {
    let (supervisor, peer) = UnixStream::pair().unwrap();
    let reader = BufReader::new(supervisor);
    attach_client(
        reader,
        Arc::new(Mutex::new(Box::new(Vec::<u8>::new()))),
        clients,
        &Arc::new(Mutex::new(VecDeque::new())),
    )
    .unwrap();
    peer
}

fn clients() -> Clients {
    new_with_callback(Arc::new(|_| {}))
}

fn clients_with_changes() -> (Clients, Arc<Mutex<Vec<PresentationSnapshot>>>) {
    let changes = Arc::new(Mutex::new(Vec::new()));
    let captured = changes.clone();
    let clients = new_with_callback(Arc::new(move |presentation| {
        captured.lock().unwrap().push(presentation);
    }));
    (clients, changes)
}

fn wait_for(clients: &Clients, attached_clients: u64, attachment_epoch: u64) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        let current = snapshot(clients);
        if current.attached_clients == attached_clients
            && current.attachment_epoch == attachment_epoch
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let current = snapshot(clients);
    assert_eq!(current.attached_clients, attached_clients);
    assert_eq!(current.attachment_epoch, attachment_epoch);
}

#[test]
fn attach_and_detach_advance_the_epoch() {
    let (clients, changes) = clients_with_changes();
    let initial = snapshot(&clients);
    assert_eq!(initial.attached_clients, 0);
    assert_eq!(initial.attachment_epoch, 0);

    let peer = attach(&clients);
    let attached = snapshot(&clients);
    assert_eq!(attached.attached_clients, 1);
    assert_eq!(attached.attachment_epoch, 1);

    drop(peer);
    wait_for(&clients, 0, 2);
    let changes = changes.lock().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(
        (changes[0].attached_clients, changes[0].attachment_epoch),
        (1, 1)
    );
    assert_eq!(
        (changes[1].attached_clients, changes[1].attachment_epoch),
        (0, 2)
    );
}

#[test]
fn fanout_pruning_emits_a_detach_edge() {
    let (clients, changes) = clients_with_changes();
    let peer = attach(&clients);
    peer.shutdown(std::net::Shutdown::Both).unwrap();

    fanout(&clients, b"output");

    wait_for(&clients, 0, 2);
    assert_eq!(
        changes.lock().unwrap().last().copied(),
        Some(snapshot(&clients))
    );
}

#[test]
fn conditional_kill_is_fenced_by_client_count_and_epoch() {
    let clients = clients();
    let kills = AtomicUsize::new(0);
    let killed = kill_if_headless(&clients, 0, |_| {
        kills.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(matches!(killed, ConditionalKillOutcome::Killed { .. }));

    let peer = attach(&clients);
    let changed = kill_if_headless(&clients, 0, |_| {
        kills.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(matches!(
        changed,
        ConditionalKillOutcome::PresentationChanged { .. }
    ));
    assert_eq!(kills.load(Ordering::SeqCst), 1);

    drop(peer);
    wait_for(&clients, 0, 2);
    let stale = kill_if_headless(&clients, 1, |_| {
        kills.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(matches!(
        stale,
        ConditionalKillOutcome::PresentationChanged { .. }
    ));
    assert_eq!(kills.load(Ordering::SeqCst), 1);

    let current = kill_if_headless(&clients, 2, |_| {
        kills.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(matches!(current, ConditionalKillOutcome::Killed { .. }));
    assert_eq!(kills.load(Ordering::SeqCst), 2);
}
