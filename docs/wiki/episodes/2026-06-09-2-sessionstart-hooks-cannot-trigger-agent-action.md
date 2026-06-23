---
type: episode-card
date: 2026-06-09
session: 3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6.jsonl
salience: root-cause
status: active
subjects:
  - hook-delivery-semantics
  - session-start-limitation
supersedes: []
related_claims: []
source_lines:
  - 628-712
captured_at: 2026-06-12T19:54:14Z
---

# Episode: SessionStart hooks cannot trigger agent action — instruction moved to UserPromptSubmit

## Prior State

The `wait-for-mention` instruction was placed in the SessionStart hook, assuming the agent would see and act on stdout output injected at session start.

## Trigger

User tested and observed nothing happening — the welcome screen showed with no LLM call, no command execution. Empirical finding: SessionStart hook stdout reaches model context but the agent is idle (no active turn) and cannot execute commands until the first user prompt.

## Decision

Moved the `wait-for-mention` instruction from SessionStart to UserPromptSubmit, gated by a once-per-session flag file (`/tmp/tenex-edge-wait-instructed-{sid}`) so it fires only on the first prompt. Applied across all three harness integrations (Claude Code, Codex, OpenCode).

## Consequences

- SessionStart hooks are architecturally limited to non-LLM side effects (publishing identity, starting the engine); any instruction requiring the agent to execute a command must go through UserPromptSubmit
- The once-per-session flag file prevents re-injection on every prompt turn
- All three harness integrations (claude-code, codex, opencode) now deliver the instruction consistently on the first active turn

## Open Tail

- Flag file in /tmp could survive across crashes, suppressing the instruction on a genuinely new session if the session ID coincidentally matches — low probability but not impossible

## Evidence

- transcript lines 628-712

