---
type: episode-card
date: 2026-06-09
session: 2cee1bc6-0f1a-4746-9de6-68ca1a7e2737
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/2cee1bc6-0f1a-4746-9de6-68ca1a7e2737.jsonl
salience: architecture
status: active
subjects:
  - turn-start-output
  - turn-check
  - hook-subcommand
  - hostdef-registry
supersedes: []
related_claims: []
source_lines:
  - 516-520
  - 1373-1816
captured_at: 2026-06-12T19:57:01Z
---

# Episode: Hook integration logic moved from Python scripts into Rust binary with data-driven HostDef registry

## Prior State

Python wrapper scripts (te-hook.py) owned context injection logic: they called turn-start fire-and-forget, then separately called inbox + who and stitched blocks together with flag files. turn_start in Rust was a no-output sync function that only marked the session working. Adding a new agent harness meant writing a new Python script.

## Trigger

User directive: 'the way turn-start command works is wrong — it should inject the stuff the agent is supposed to see, not leave that up to the wrapper script' and 'that logic shouldn't be parked in scripts' and 'do it — make sure is well architected so it doesn't blow in complexity when we have support for 50 agent harnesses'

## Decision

All hook dispatch absorbed into the Rust binary via a single `tenex-edge hook --host <name> --type <hook>` subcommand backed by a data-driven HostDef registry (one struct per harness, zero new code). turn_start became async: on first turn emits full roster + wait-for-mention hint; on subsequent turns emits only deltas (new peers since prev_turn_started_at, inbox drains, status changes). New TurnCheck command (sync, read-only peek_inbox, no state.db writes) powers PostToolUse mid-run checks. Python scripts deleted entirely.

## Consequences

- Zero Python dependency for any agent integration
- Adding a new harness = one HostDef struct with field mappings, no new dispatch code
- turn_check is pure-read (no new concurrent writers to state.db), safe for PostToolUse frequency
- peer_sessions.first_seen column added to state.db (migration included) to enable accurate delta roster
- Codex config template now references __BIN__ directly; Claude Code template references tenex-edge on PATH
- PostToolUse hook added to Codex config; Claude Code PostToolUse deferred (output format unverified)

## Open Tail

- Claude Code PostToolUse hook format needs verification before wiring
- OpenCode and other harnesses can be added as HostDef entries

## Evidence

- transcript lines 516-520
- transcript lines 1373-1816

