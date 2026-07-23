use super::*;
use crate::state::{AdmittedRuntimeFacts, RegisterSession, StopReason};

fn seed_retained(store: &Store) {
    store.upsert_channel("proj", "proj", "", "", 900).unwrap();
    store
        .upsert_profile_with_agent_slug(
            "pk-codex",
            "willow-summit-042-codex",
            "willow-summit-042",
            "codex",
            "laptop",
            false,
            900,
        )
        .unwrap();
    let generation = store
        .reserve_session_with_facts(
            &RegisterSession {
                pubkey: "pk-codex".into(),
                observed_harness: "codex".into(),
                agent_slug: "codex".into(),
                channel_h: "proj".into(),
                child_pid: None,
                now: 900,
            },
            &AdmittedRuntimeFacts {
                observed_harness: "codex".into(),
                claimed_harness: String::new(),
                bundle: "codex-acp".into(),
                transport: "app-server".into(),
                endpoint_provenance: "launch".into(),
            },
        )
        .unwrap();
    let session = store.get_session("pk-codex").unwrap().unwrap();
    assert_eq!(
        store
            .commit_confirmed_session_admission(
                "pk-codex",
                "proj",
                generation,
                session.lifecycle_epoch,
                900,
            )
            .unwrap(),
        crate::state::ConfirmedAdmissionCommit::Committed
    );
    store
        .mark_runtime_stopped_if_generation("pk-codex", generation, StopReason::HeadlessExit, 900)
        .unwrap();
}

#[test]
fn who_snapshot_renders_retained_standing_as_dormant_presence() {
    let store = Store::open_memory().unwrap();
    seed_retained(&store);

    let snapshot = load_who_snapshot(&store, Some("proj"), 1_000, "laptop").unwrap();
    let row = snapshot.rows.first().expect("dormant row");
    assert!(row.dormant);
    assert_eq!(row.slug, "codex");
    assert_eq!(row.age_secs, Some(100));
    assert!(!row.remote);
}

#[test]
fn who_snapshot_hides_expired_retention() {
    let store = Store::open_memory().unwrap();
    seed_retained(&store);
    let snapshot = load_who_snapshot(&store, Some("proj"), 4_501, "laptop").unwrap();
    assert!(snapshot.rows.is_empty());
}
