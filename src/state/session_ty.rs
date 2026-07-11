/// A local agent process THIS daemon hosts. OS handles only — never agent
/// identity (that lives in `relay_status`/`relay_profiles`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub session_id: String,
    pub agent_pubkey: String,
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
    pub last_distill_at: u64,
    /// Explicit, broad work topic set by the agent itself. It is intentionally
    /// separate from the automatic title/activity distillation state.
    pub work_topic: String,
    /// Seconds when [`Self::work_topic`] was explicitly set. Distillation pauses
    /// for the first 30 minutes; only then does hook context display the topic.
    pub work_topic_set_at: u64,
    pub seen_cursor: u64,
    pub title: String,
    pub activity: String,
    pub resume_id: String,
    /// Consecutive failed status-title generation attempts (reset to 0 on the
    /// next success). Gates the throttled agent-facing heads-up in
    /// `turn_context::start`.
    pub distill_fail_streak: u64,
    /// When the agent-facing heads-up about failing status-title generation
    /// was last injected, so it repeats at most a few times per hour rather
    /// than every turn.
    pub distill_notice_at: u64,
    /// First successful explicit channel publish by this session. Once set,
    /// PTY mention delivery no longer arms turn-end auto-publish for it.
    pub explicit_chat_published_at: u64,
}

impl Session {
    pub fn work_topic_suppresses_distillation(&self, now: u64) -> bool {
        crate::work_topic::suppresses_distillation(&self.work_topic, self.work_topic_set_at, now)
    }

    pub fn visible_work_topic(&self, now: u64) -> Option<&str> {
        crate::work_topic::is_visible(&self.work_topic, self.work_topic_set_at, now)
            .then_some(self.work_topic.as_str())
    }
}
