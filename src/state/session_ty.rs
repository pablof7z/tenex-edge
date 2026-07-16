/// One local agent runtime hosted by this daemon. `pubkey` is its sole identity;
/// `runtime_generation` only fences stale asynchronous lifecycle callbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub pubkey: String,
    pub runtime_generation: u64,
    pub agent_slug: String,
    pub channel_h: String,
    pub harness: String,
    pub child_pid: Option<i32>,
    pub transcript_path: Option<String>,
    pub alive: bool,
    pub created_at: u64,
    pub last_seen: u64,
    pub working: bool,
    pub turn_started_at: u64,
    pub seen_cursor: u64,
    pub title: String,
    pub explicit_chat_published_at: u64,
}
