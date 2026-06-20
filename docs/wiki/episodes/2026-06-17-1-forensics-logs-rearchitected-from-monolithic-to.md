---
type: episode-card
date: 2026-06-17
session: 3b87cdd2-dc84-40d5-9bf0-677e282fe0e4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/3b87cdd2-dc84-40d5-9bf0-677e282fe0e4.jsonl
salience: architecture
status: active
subjects:
  - forensics-log-storage
  - hook-tail-data-path
  - per-session-jsonl
supersedes: []
related_claims: []
source_lines:
  - 172-178
  - 598-656
  - 728-861
captured_at: 2026-06-18T00:53:59Z
---

# Episode: Forensics logs rearchitected from monolithic to per-session layout

## Prior State

All hook and command events appended to two global files (hook-calls.jsonl 101MB, command-calls.jsonl 369MB). TUI read the entire files on every 1s refresh cycle via read_to_string, causing 10–20s freezes. Tail-read truncation at 2MB (later 20MB) still risked cutting off session-start events for active sessions.

## Trigger

User reported hook-tail taking 10–20s per interaction. Diagnosis: read_to_string on 369MB+ files every refresh + blocking call() that could spawn a daemon (30s stall). User then asked whether logs should be per-session, confirming the monolithic layout was the root problem.

## Decision

New directory layout: ~/.tenex/edge/sessions/<session-id>/hook-calls.jsonl and command-calls.jsonl. Writers (hook_forensics.rs, command_forensics.rs) extract session_id at write time and route to per-session files. _unscoped/ fallback for entries lacking a session_id. Env-var overrides (TENEX_EDGE_HOOK_CALL_LOG, TENEX_EDGE_COMMAND_CALL_LOG) still honored. Reader (debug.rs) enumerates sessions/*/ directories, reads whole small files without byte-limit truncation, falls back to legacy monolithic paths for backward compat.

## Consequences

- Per-session files stay small — no tail truncation needed, early events always present
- Natural pruning: delete a directory to purge a session's history
- TUI only reads relevant sessions instead of parsing 130K+ lines from all sessions
- Background thread + call_no_spawn also keep TUI responsive during loads
- Old monolithic files deleted; new sessions write exclusively to per-session directories

## Open Tail

- Could add tenex-edge debug prune --older-than for automatic cleanup
- Legacy monolithic fallback path can be removed once migration is confirmed

## Evidence

- transcript lines 172-178
- transcript lines 598-656
- transcript lines 728-861

