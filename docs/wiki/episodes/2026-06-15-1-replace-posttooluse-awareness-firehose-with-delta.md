---
type: episode-card
date: 2026-06-15
session: a0037729-ad51-460a-880d-0a9699f6ee41
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a0037729-ad51-460a-880d-0a9699f6ee41.jsonl
salience: architecture
status: superseded
subjects:
  - posttooluse-awareness
  - sibling-delta
  - turn-check-cursor
supersedes:
  - 2026-06-15-1-delta-gated-posttooluse-sibling-awareness-replaces
related_claims: []
source_lines:
  - 1-51
  - 521-583
  - 589-627
  - 687-703
  - 1193-1211
  - 1364-1462
captured_at: 2026-06-15T09:54:19Z
---

# Episode: Replace PostToolUse awareness firehose with delta-gated sibling awareness

## Prior State

pc awareness --hook PostToolUse fired on every tool call, dumping the entire cross-repo roster (all sessions, all projects, including 'just started' and 'finished' entries). It was global, ungated, repeated identically dozens of times per turn, and produced the noise the user called 'pointless.'

## Trigger

User complained the PostToolUse hook output was 'incredibly noisy and pointless' — every tool call injected the full multi-project session roster into context.

## Decision

Replace pc awareness PostToolUse with tenex-edge hook --type post-tool-use implementing three kill-noise rules: (1) delta-gated — per-session cursor (turn_state.last_check_at) ensures each fact surfaces at most once; nothing emitted when nothing changed; (2) project-scoped — only sessions in this project, never cross-repo firehose; (3) self-excluded — calling session never echoes its own churn. 60s debounce floor between mid-turn checks (first check of a turn always fires). Output includes title + activity + idle marker, rendered as 6-char session_short_code (not raw UUID).

## Consequences

- New schema column turn_state.last_check_at with turn_check_due() gating method; cursor reset on turn start, advanced on each check that runs
- list_status_changes_since tuple expanded to include activity field (breaking API change for all callers)
- New EmitFormat enum: PostToolUse requires {"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"..."}} JSON envelope (plain stdout is silently ignored by Claude Code); UserPromptSubmit/opencode remain plain text
- pc awareness PostToolUse hook replaced in ~/.claude/settings.json; other pc hooks (inject, capture, session_start, statusline) untouched
- Session IDs must use session_short_code() — not pubkey_short() and not raw UUID — to match who/tmux display and remain copy-pasteable into send --to
- Daemon must cycle to pick up new rpc_turn_check logic; old daemon binary serves inbox-peek-only until restart

## Open Tail

- Running daemon (from previous day) still serves old turn_check; delta half inert until daemon restarts
- End-to-end smoke test blocked by SIGKILL (137) in Claude Code Bash environment — unit-test coverage only

## Evidence

- transcript lines 1-51
- transcript lines 521-583
- transcript lines 589-627
- transcript lines 687-703
- transcript lines 1193-1211
- transcript lines 1364-1462

