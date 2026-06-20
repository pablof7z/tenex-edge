---
type: episode-card
date: 2026-06-09
session: 2cee1bc6-0f1a-4746-9de6-68ca1a7e2737
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/2cee1bc6-0f1a-4746-9de6-68ca1a7e2737.jsonl
salience: architecture
status: superseded
subjects:
  - turn-start-context-injection
  - hook-subcommand
  - hostdef-registry
  - turn-check
supersedes: []
related_claims: []
source_lines:
  - 516-521
  - 821-1031
  - 1130-1167
  - 1359-1373
  - 1394-1685
captured_at: 2026-06-17T23:44:19Z
---

# Episode: Context injection moves from Python scripts into Rust binary; scripts eliminated

## Prior State

Context injection (inbox draining, peer presence, wait-for-mention hint) was assembled in Python wrapper scripts by calling multiple tenex-edge CLI commands (turn-start, inbox, who) and stitching output. Scripts also managed flag files for first-turn detection and ANSI stripping.

## Trigger

User directive: 'the way turn-start command works is wrong — it should inject the stuff the agent is supposed to see, not leave that up to the wrapper script! … that logic shouldn't be parked in scripts'. Follow-up: 'do we even need these scripts?'

## Decision

Context injection logic moved entirely into the Rust binary. turn_start is now async and outputs full context on first turn (full roster + wait-for-mention hint) and deltas on subsequent turns (new peers, status changes, inbox). New turn_check command is read-only (no state.db writes) for PostToolUse mid-run checks. Python scripts eliminated; a hook subcommand with a data-driven HostDef registry replaces them — adding a new agent harness requires one struct, zero new code.

## Consequences

- turn_start now async: fetches mentions from relay, drains inbox, renders peer deltas — scripts just forward its stdout
- turn_check is pure-read (peek_inbox) to avoid adding concurrent writers to SQLite state.db
- First turn emits full roster + wait-for-mention hint; subsequent turns emit only deltas using peer_sessions.first_seen column and status changes since prev_turn_started_at
- HostDef registry in Rust: name, agent_slug, session_id_fields, transcript_field, output_format (PlainText vs JsonSystemMessage), pid_search — scales to 50+ harnesses with zero code branches
- No Python dependency anywhere in the integration; config templates reference the binary directly via __BIN__ placeholder or PATH
- Codex PostToolUse hook wired in config.template.toml; Claude Code PostToolUse deliberately skipped (stdout format for that hook unverified)

## Open Tail

- Claude Code PostToolUse hook format needs verification before wiring turn-check there

## Evidence

- transcript lines 516-521
- transcript lines 821-1031
- transcript lines 1130-1167
- transcript lines 1359-1373
- transcript lines 1394-1685

