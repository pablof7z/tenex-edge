---
type: episode-card
date: 2026-06-09
session: 9ac666e5-b468-4af2-be5e-83e5c8f2d1d2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9ac666e5-b468-4af2-be5e-83e5c8f2d1d2.jsonl
salience: architecture
status: active
subjects:
  - cli-lifecycle-verbs
  - hook-dispatch
  - opencode-integration
supersedes:
  - 2026-06-09-1-activity-distillation-replaced-tool-driven-turn
related_claims: []
source_lines:
  - 24-26
  - 253-257
  - 290-294
  - 296-298
  - 396-410
  - 536-543
  - 977-1001
captured_at: 2026-06-17T23:55:18Z
---

# Episode: CLI lifecycle verbs removed; hook becomes sole harness entry point

## Prior State

The CLI exposed session-start, session-end, turn-start, turn-check, and turn-end as first-class subcommands. Help text implied harnesses call them directly. The hook subcommand existed but was just one path among many. The opencode integration called the bare verbs programmatically (not via hook).

## Trigger

User observed that most CLI commands were superseded by `tenex-edge hook`, then explicitly directed that if the verbs are internal they should not be in the CLI at all — hook should be the single entry point.

## Decision

Removed all five session/turn lifecycle verbs (session-start, session-end, turn-start, turn-check, turn-end) from the CLI enum and dispatch. They are now private functions that only hook_run calls. Migrated the opencode integration (the sole harness still using bare verbs) to pipe JSON payloads through `hook`. Added two hook capabilities for opencode: explicit `pid` in the payload and a `generates_sid` HostDef flag (gated to opencode only — empty session-id for Claude Code/Codex remains a fail-open no-op to avoid orphan sessions).

## Consequences

- All three harnesses (Claude Code, Codex, opencode) now route through `tenex-edge hook --host X --type Y`; no harness calls a bare lifecycle verb.
- The `generates_sid` flag is deliberately NOT universal: for real harnesses an empty session-id means a malformed payload, not a request to generate one.
- The daemon smoke test was rewritten to drive `hook` over stdin instead of calling bare verbs.
- inbox, who, send-message, wait-for-mention remain as CLI commands — they are manual/agent-facing, not hook-driven.
- Design docs (M1.md, daemon-design.md) still reference session_start etc. as conceptual RPCs — only the CLI verbs were removed, not the daemon RPCs.

## Open Tail

- Design docs still mention the removed verb names as conceptual steps/RPCs — may need a sweep to clarify that only the CLI surface changed.
- The untracked daemon WIP (src/daemon/server.rs, tests/daemon_integration.rs) was edited concurrently by other agents; the migrated test and channel README were committed separately to avoid clobbering.

## Evidence

- transcript lines 24-26
- transcript lines 253-257
- transcript lines 290-294
- transcript lines 296-298
- transcript lines 396-410
- transcript lines 536-543
- transcript lines 977-1001

