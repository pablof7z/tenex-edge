---
type: episode-card
date: 2026-06-09
session: 9ac666e5-b468-4af2-be5e-83e5c8f2d1d2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9ac666e5-b468-4af2-be5e-83e5c8f2d1d2.jsonl
salience: architecture
status: active
subjects:
  - cli-surface
  - hook-command
  - host-integration-opencode
supersedes: []
related_claims: []
source_lines:
  - 24-25
  - 255-258
  - 286-295
  - 357-360
  - 505-506
  - 554-593
  - 979-991
captured_at: 2026-06-12T20:09:36Z
---

# Episode: CLI lifecycle verbs removed; hook becomes sole host-facing entry point

## Prior State

The CLI exposed session-start, session-end, turn-start, turn-check, and turn-end as top-level subcommands alongside `hook`. Help text implied harnesses called the bare verbs directly (e.g., 'used by the turn-start hook'). The opencode integration drove lifecycle by calling the bare verbs programmatically rather than through hook.

## Trigger

User observed that since hook already internally dispatches to these functions, exposing them as CLI subcommands contradicts the design: 'if they are internal why are they in the CLI? they should not be there at all and tenex-edge hook should just internally call them.'

## Decision

Remove all five lifecycle verbs (session-start, session-end, turn-start, turn-check, turn-end) from the CLI enum and dispatch. They remain as private functions called only by hook_run. Migrate the opencode plugin to pipe JSON payloads to `tenex-edge hook --host opencode` and capture stdout. Add a `generates_sid` HostDef flag (gated to opencode only) so that an empty session-id causes the daemon to generate and print a SID — Claude Code/Codex still fail-open on empty IDs to avoid spawning orphan sessions. The `pid` field is also now accepted in hook payloads for harnesses that know their own PID.

## Consequences

- CLI surface reduced: hosts now have a single integration pattern (`tenex-edge hook --host X --type Y`)
- opencode plugin rewritten from direct CLI calls to stdin-JSON hook invocations
- New `generates_sid` HostDef flag controls SID generation — must be explicitly enabled per host to prevent malformed payloads from creating orphan sessions
- Daemon smoke test migrated from bare-verb invocation to hook-over-stdin, exercising the generate-and-print SID branch
- Manual-facing commands (inbox, who, send-message, wait-for-mention, tail, acl, project, doctor) remain as CLI subcommands
- README command table and channel/README manual-harness docs updated to reflect hook-only lifecycle

## Open Tail

- Design docs (M1.md, docs/daemon-design.md) still reference session-start etc. as conceptual RPCs — these remain valid since the daemon RPCs still exist, only the CLI verbs were removed

## Evidence

- transcript lines 24-25
- transcript lines 255-258
- transcript lines 286-295
- transcript lines 357-360
- transcript lines 505-506
- transcript lines 554-593
- transcript lines 979-991

