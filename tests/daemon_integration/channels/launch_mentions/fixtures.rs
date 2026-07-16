use super::*;
use mosaico::state::RegisterSession;

pub(super) fn seed_dormant_local_session(home: &Home, channel: &str, agent: &str) -> String {
    let pubkey = Keys::generate().public_key().to_hex();
    let store = Store::open(&home.store_path()).unwrap();
    store
        .reserve_session(&RegisterSession {
            pubkey: pubkey.clone(),
            harness: "opencode".to_string(),
            agent_slug: agent.to_string(),
            channel_h: channel.to_string(),
            child_pid: None,
            transcript_path: None,
            now: 1,
        })
        .expect("seed dormant local session");
    store
        .mark_dead(&pubkey)
        .expect("mark seeded local session dead");
    store
        .upsert_profile_with_agent_slug(&pubkey, agent, agent, agent, "test-host", false, 1)
        .expect("seed offline profile");
    pubkey
}
