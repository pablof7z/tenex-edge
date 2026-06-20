---
type: episode-card
date: 2026-06-14
session: 55a2eb41-5ff1-4eb3-bdb8-7a4728422be5
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/55a2eb41-5ff1-4eb3-bdb8-7a4728422be5.jsonl
salience: root-cause
status: active
subjects:
  - opencode-tenex-edge-plugin
  - tenex-edge-hook-interface
supersedes: []
related_claims: []
source_lines:
  - 108-166
captured_at: 2026-06-18T00:17:33Z
---

# Episode: opencode tenex-edge plugin was stale and silently broken — updated to unified hook interface

## Prior State

The installed opencode tenex-edge plugin called old subcommands (session-start, turn-start, turn-end) that no longer exist in the CLI. The plugin also swallowed exec errors, so opencode sessions silently failed to register on the fabric — no presence, no distillation, no idle marking.

## Trigger

Investigation of opencode plugin revealed it was outdated (9521 bytes installed vs 10969 canonical) and calling removed subcommands. CLI help confirmed session-start/turn-start/turn-end no longer exist — everything routes through the unified `tenex-edge hook --host <name> --type <t>` entry point.

## Decision

Replaced the stale plugin with the canonical integrations/opencode/tenex-edge.ts that uses the unified `hook` interface (piping JSON on stdin for session-start, user-prompt-submit, stop), verified byte-identical to the repo copy.

## Consequences

- New opencode sessions will properly register on the tenex-edge fabric (presence, distillation, idle marking)
- Currently-running opencode sessions still use the old broken plugin until restarted — opencode loads plugins at startup
- Claude Code and Codex were already correctly configured with the unified hook interface; only opencode lagged behind

## Open Tail

- Existing opencode sessions need restart to pick up the fix

## Evidence

- transcript lines 108-166

