---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: product
status: active
subjects:
  - cli-surface
  - inbox-new-session
  - tmux-spawn-removal
supersedes: []
related_claims: []
source_lines:
  - 411-413
  - 615-707
captured_at: 2026-06-16T10:37:51Z
---

# Episode: Move session spawn CLI from tmux to inbox new-session

## Prior State

The only CLI path to spawn a new agent session was `tenex-edge tmux spawn --agent <slug>`, a tmux subsystem command — conceptually coupled to the tmux control plane rather than the agent communication surface.

## Trigger

User's mockup referenced `tenex-edge inbox new-session --agent` as the command to show agents. When the assistant assumed it didn't exist and used `tmux spawn` instead, the user corrected: "for fuck's sake! tenex-edge inbox new-session --agent is a new fucking command" — making clear this was a new product surface to build, not a misunderstanding.

## Decision

Created `tenex-edge inbox new-session --agent <slug> [--project <slug>]` as a new `InboxAction` subcommand, dispatching through `messaging::new_session()` to the same daemon `tmux_spawn` RPC. Removed the `tenex-edge tmux spawn` CLI subcommand entirely. The daemon RPC and the interactive TUI spawn path remain unchanged.

## Consequences

- Session spawning is now surfaced under `inbox`, aligned with the agent communication domain — more discoverable for both agents reading `who` output and humans
- The `tmux spawn` CLI subcommand is gone (`error: unrecognized subcommand 'spawn'`)
- Agent-facing `who` renderer documents `tenex-edge inbox new-session --agent <slug>` as the canonical instruction
- Underlying implementation unchanged: same daemon RPC, same TUI path — only the CLI surface moved

## Open Tail

*(none)*

## Evidence

- transcript lines 411-413
- transcript lines 615-707

