---
type: episode-card
date: 2026-06-09
session: 956595fb-fa6a-45f8-869c-b53cae16124f
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/956595fb-fa6a-45f8-869c-b53cae16124f.jsonl
salience: reversal
status: active
subjects:
  - activity-distillation
  - turn-lifecycle
  - heuristic-distiller-removal
  - tool-event-removal
supersedes:
  - 2026-06-08-1-agent-status-distillation-transcript-first-native
related_claims: []
source_lines:
  - 1-8
  - 13-21
  - 287-291
  - 560-570
  - 620-643
captured_at: 2026-06-12T19:55:32Z
---

# Episode: Activity distillation: from tool-driven to turn-driven transcript-only model

## Prior State

Activity distillation was coupled to tool-use events end-to-end: PostToolUse hook → observe CLI verb → ToolEvent rows in obs table → drain_obs → burst buffer gated by distill_min → HeuristicDistiller (offline fallback) or LLM distiller. Even the LLM path took ToolEvent as input; the transcript was bolted on as supplementary context. The observe help text mentioned tool use because tool calls were the trigger, gate, and fallback input.

## Trigger

User's explicit directive: 'tool use is NOT what we observe to set the activity an agent is working on; it's the transcript of the conversation.' Directed a turn-based model where a 30s poll (later refined to: first check at 30s, then every 5 min) inspects the transcript only, with turn-start/turn-end hooks bracketing the agent's work cycle. No heuristic fallback — LLM-only. User confirmed: no HeuristicDistiller, no-LLM fallback (line 288).

## Decision

Replaced the entire tool-driven pipeline with a turn-driven model: (1) Removed observe verb, ToolEvent struct, HeuristicDistiller, obs table, record_obs/drain_obs, llm_available gate. (2) Added turn-start --session [--transcript] and turn-end --session CLI verbs that flip a turn_state(session_id, working, turn_started_at) table. (3) Engine polls turn_state on its existing tick; if working and turn_started_at ≥ 30s ago (and not yet distilled), reads the conversation transcript and calls the LLM distiller. Re-distills every 5 min for long turns. Turn-end clears working and publishes idle Status. (4) Status (NIP-38, 90s TTL) is refreshed on the 30s heartbeat from engine-local state, not only at distill time — preventing TTL expiry between re-distill cycles.

## Consequences

- Short turns (<30s) cost zero LLM calls — the delay itself is the cost gate, replacing the tool-burst heuristic
- HeuristicDistiller and all no-LLM fallback paths removed; edge-distillation model in llms.json is required for activity to publish
- All three host integrations rewired: Claude Code (UserPromptSubmit→turn-start, Stop→turn-end), Codex (UserPromptSubmit→turn-start, Stop→turn-end with transcript_path), OpenCode (transform→turn-start gated on new user-msg id, session.idle→turn-end, transcript snapshot refreshed on tool.execute.after)
- distill_activity() is now transcript-only; the &ToolEvent parameter and tools input are gone
- PostToolUse hook removed from Claude Code settings.template.json and global settings; observe CLI verb removed entirely
- OpenCode's session.idle→turn-end assumption needs live verification (may fire between agentic round-trips, going idle early)

## Open Tail

- OpenCode session.idle per-turn assumption unverified in live use — could cause premature idle on long turns
- send-message CLI positional-arg mismatch exists in injected hint text across integrations (flagged, out of scope)

## Evidence

- transcript lines 1-8
- transcript lines 13-21
- transcript lines 287-291
- transcript lines 560-570
- transcript lines 620-643

