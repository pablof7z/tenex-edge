---
type: episode-card
date: 2026-06-09
session: 956595fb-fa6a-45f8-869c-b53cae16124f
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/956595fb-fa6a-45f8-869c-b53cae16124f.jsonl
salience: reversal
status: superseded
subjects:
  - activity-distillation
  - turn-lifecycle
  - heuristic-distiller-removal
supersedes:
  - 2026-06-08-1-agent-status-distilled-from-conversation-transcript
related_claims: []
source_lines:
  - 1-9
  - 287-291
  - 568-570
  - 582-593
  - 596-603
  - 1220-1236
captured_at: 2026-06-17T23:43:10Z
---

# Episode: Activity distillation replaced: tool-driven → turn-driven, transcript-only

## Prior State

Activity distillation was triggered by PostToolUse hooks, accumulated ToolEvent rows in an obs table, and gated by a minimum-burst interval (distill_min). The HeuristicDistiller (tool-name-based) was the always-available fallback. The observe CLI verb, ToolEvent struct, obs table, and HeuristicDistiller formed the entire pipeline. Transcript reading was bolted on as LLM context but the system was keyed off 'a tool happened'.

## Trigger

User identified that distillation was 'way too coupled to tool use' and proposed replacing the entire tool-event pipeline with a turn-lifecycle model: UserPromptSubmit→turn-start arms a 30s timer; if the turn is still running at 30s, the engine reads the transcript and LLM-distills intent; Stop→turn-end clears the turn. Short turns (<30s) cost zero LLM calls. User also directed: no HeuristicDistiller, no no-LLM fallback, distillation is LLM-only.

## Decision

Removed the entire tool-driven pipeline (observe verb, ToolEvent, HeuristicDistiller, obs table, record_obs/drain_obs, distill_min gate, llm_available) and replaced it with a turn-lifecycle model: turn-start/turn-end CLI verbs flip a turn_state(session_id, working, turn_started_at) table. The per-session engine polls turn state and distills from the conversation transcript (LLM-only) on cadence: first at 30s, then every 5 minutes. turn-end marks idle and publishes idle Status. Hooks remapped: UserPromptSubmit→turn-start, Stop→turn-end (PostToolUse→observe removed). All three host integrations (Claude Code, Codex, OpenCode) rewired.

## Consequences

- Short turns (<30s threshold) produce zero LLM calls — the delay IS the cost gate
- HeuristicDistiller and all no-LLM fallbacks removed; LLM config is now required for activity distillation
- Status TTL (90s) would blink idle during 5-min re-distill intervals → Status is now refreshed on the 30s heartbeat from engine-local cached distilled line, decoupled from Activity note publication
- distill_activity() is now transcript-only (lost its &[ToolEvent] parameter)
- OpenCode keeps transcript snapshot fresh via tool.execute.after (re-snapshots on each tool use within the turn)
- Installed binary, hook script, and global ~/.claude/settings.json all required updating to remove dead PostToolUse→observe wiring
- macOS reinstall requires temp-file + ad-hoc re-sign + atomic mv to avoid SIGKILL from AMFI

## Open Tail

- OpenCode's session.idle→turn-end assumes per-turn firing; if it fires mid-loop during a long turn, the turn would go idle prematurely — needs live confirmation
- Pre-existing send-message flag syntax bug (positional vs --recipient/--message) exists in injected hint text across integrations

## Evidence

- transcript lines 1-9
- transcript lines 287-291
- transcript lines 568-570
- transcript lines 582-593
- transcript lines 596-603
- transcript lines 1220-1236

