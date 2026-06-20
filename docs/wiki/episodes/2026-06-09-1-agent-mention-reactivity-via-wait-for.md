---
type: episode-card
date: 2026-06-09
session: 3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6.jsonl
salience: product
status: superseded
subjects:
  - wait-for-mention
  - agent-reactivity
  - hook-injection
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 133-168
  - 170-176
  - 260-260
  - 628-670
  - 695-711
captured_at: 2026-06-17T23:41:48Z
---

# Episode: Agent mention reactivity via wait-for-mention command

## Prior State

Mentions were written to the SQLite inbox by the background engine but only surfaced to the agent on the next user prompt via the UserPromptSubmit hook. Idle agents had no mechanism to be woken on incoming mentions — they remained dormant until the next human prompt.

## Trigger

User proposed a blocking command that waits for a mention, run in the background, so the agent is woken when it completes. User also corrected the assistant's assumption that idle agents wouldn't wake on background process completion — empirically verified that they do across harnesses.

## Decision

Implemented `tenex-edge wait-for-mention` as a CLI subcommand that polls the SQLite inbox every 500ms, prints received mentions on exit, and reminds the agent to re-run. The instruction to run this command is injected via the UserPromptSubmit hook (gated once-per-session with a flag file), NOT via SessionStart — because SessionStart fires before any LLM turn, so the agent cannot execute commands at that point.

## Consequences

- Idle agents are now woken when a mention arrives, enabling true async inter-agent communication
- The command doubles as a coordination primitive for active/autonomous agents blocking on a peer response
- Injection must use UserPromptSubmit with a once-per-session flag file; SessionStart injection is ineffective because no LLM turn exists to act on it
- All three harnesses (Claude Code, Codex, OpenCode) updated with the instruction
- Hook scripts point directly to the source tree (no deployed copies) to prevent future divergence

## Open Tail

- The once-per-session flag file approach needs testing across fresh sessions to confirm it resets properly
- OpenCode harness uses a one-time hint block rather than the flag-file pattern — may need convergence testing

## Evidence

- transcript lines 1-1
- transcript lines 133-168
- transcript lines 170-176
- transcript lines 260-260
- transcript lines 628-670
- transcript lines 695-711

