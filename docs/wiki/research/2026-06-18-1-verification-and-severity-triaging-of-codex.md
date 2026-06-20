---
type: research-record
date: 2026-06-18
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
source_lines: 1962-2032
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Verification and severity-triaging of codex review findings on session-state rearchitecture, confirming critical harness-id vs canonical-id alias mismatch and heartbeat expiration bugs"
captured_at: 2026-06-18T00:44:46Z
---

Verification and severity-triaging of codex review findings on session-state rearchitecture, confirming critical harness-id vs canonical-id alias mismatch and heartbeat expiration bugs

---

Assistant: Codex found **serious architectural gaps** — the code compiles and tests pass but doesn't actually deliver the design. This is exactly why the review mattered. Let me verify the top findings against the real code before trusting them (codex can be wrong too):

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== #1 rpc_turn_start: passes raw p.session to start_turn? ===
        serde_json::from_value(params.clone()).context("parsing turn_start params")?;
    if p.session.is_empty() {
        return Ok(serde_json::json!({ "context": serde_json::Value::Null }));
    }

    let prev_started = state.with_store(|s| {
        let (_, prev) = s.get_turn_state(&p.session).unwrap_or((false, 0));
        // Canonical transition: busy=1, turn_id+1, activity cleared, version bump +
        // status_outbox enqueue. Also writes turn_state so turn_check_due() works.
        s.start_turn(&p.session, now_secs()).ok();
        if let Some(path) = p.transcript.as_deref().filter(|x| !x.is_empty()) {
            s.set_session_transcript(&p.session, path).ok();
            // Snapshot the last assistant text so rpc_turn_end can poll until a
            // *new* (different) response appears — Claude Code writes the
            // transcript after the stop hook f

Assistant: Finding #1 confirmed at the call site — `start_turn(&p.session)` uses the raw harness id. Whether it's fatal depends on whether the canonical id equals the harness id for claude/codex. Let me check the minting logic and the heartbeat:

User: [Tool result]: chpwd:5: command not found: _iterm2_set_profile
=== register_or_reassert_session: canonical id minted or harness id reused? ===
    pub fn register_or_reassert_session(
        &self,
        obs: &SessionObservation,
    ) -> Result<SessionSnapshot> {
        let alias_hit = self.alias_lookup(obs);
        let live = self.live_locators_for(&obs.host, &obs.project, &obs.agent_pubkey, obs)?;
        let decision = crate::session::resolve_identity(obs, alias_hit, &live);
        let id = match decision {
            IdentityDecision::Existing(id) | IdentityDecision::Reattach(id) => {
                self.reassert_session_row(id.as_str(), obs)?;
                id.into_string()
            }
            IdentityDecision::Supersede { old } => {
                self.supersede_session(old.as_str(), obs.observed_at)?;
                let id = mint_session_id();
                self.insert_session_row(&id, obs)?;
                id
            }
            IdentityDecision::Mint => {
        

Assistant: Verified — and codex is **right about the critical ones**. I confirmed the two worst myself:

- **#1 is real and fatal.** `register_or_reassert_session` always `mint_session_id()`s a fresh canonical id and stores the harness id as an *alias* (`state.rs`). But `rpc_turn_start`/`turn_end`/`session_end` call the transitions with the raw harness id (`p.session`), and `start_turn` does `UPDATE session_state WHERE session_id=?` — which matches **zero rows** for claude/codex. So the canonical row never gets busy/title/distill/status updates. The `get_session()` alias-resolves only the *context read*, masking it. **The whole aggregate is inert for claude and codex sessions** — the two main harnesses.
- **#3 is real.** `heartbeat_session` only bumps `last_seen` and deliberately doesn't enqueue; the only publisher drains the outbox. So a live idle session publishes once, then its kind:30315 **expires ~90s later despite 30s heartbeats** → live sessions show as gone. The expiration design is half-wired.

**Why the tests didn't catch this:** the integration agent rewrote the tests to capture and assert against the *canonical* id directly — bypassing the exact hook→RPC path where the alias bug lives. So "195 green" was necessary but not sufficient. My "verified green" earlier was honest but shallow; this review is what caught it.

Triaged findings (codex's, criticals personally confirmed):

| Sev | Issue | Status |
|-----|-------|--------|
| 🔴 Critical | Hooks mutate by harness id → canonical transitions no-op (claude/codex) | **confirmed** |
| 🔴 Critical | Turn start/end called by *both* RPC and runtime → double `turn_id`/version/outbox | plausible, confirmed double call site |
| 🟠 High | Heartbeat doesn't re-arm expiration → live sessions expire off relay | **confirmed** |
| 🟠 High | Upgrade doesn't backfill `session_state` from `sessions` → existing sessions invisible | confirmed earlier |
| 🟠 High | Turn-start deltas not self-excluded → session sees its own change | credible |
| 🟡 Med | `turn_check` writes the debounce cursor (not pure-read) | by-design exception, low concern |
| 🟡 Med | Outbox publishes current snapshot, not per-version → duplicate events | credible |
| 🟡 Med | Alias lookup collapses namespaces → collision routes to wrong session | credible |
| 🟡 Med | Transitions aren't real SQLite transactions → not crash-safe | credible |

**Bottom line: the refactor is on master but not actually functional for claude/codex, and live sessions expire.** It should not go to the binary cutover as-is.

Given the last autonomous round produced compiling-but-broken code, I'd fix the criticals/highs **myself, carefully, with a real end-to-end test through the hook path** (not another big fan-out). The core fix is small and surgical: resolve harness-id→canonical at the RPC boundary (one helper, applied in turn_start/turn_end/session_end), pick a single owner for turn transitions, and make heartbeat re-arm the outbox.

Want me to fix the criticals + highs now? I'd add a test that drives the actual `session_start → user-prompt-submit → stop` hook sequence and asserts the canonical row's title/busy/expiration update — the test that was missing.
