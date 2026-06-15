---
type: episode-card
date: 2026-06-15
session: a0037729-ad51-460a-880d-0a9699f6ee41
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a0037729-ad51-460a-880d-0a9699f6ee41.jsonl
salience: product
status: active
subjects:
  - post-tool-use-awareness
  - turn-check-delta
  - session-delta-rendering
supersedes:
  - 2026-06-15-1-replace-posttooluse-firehose-with-delta-gated
related_claims: []
source_lines:
  - 1-9
  - 521-577
  - 1364-1376
captured_at: 2026-06-15T09:51:04Z
---

# Episode: Delta-gated PostToolUse sibling awareness replaces global firehose

## Prior State

The `pc awareness` PostToolUse hook fired on every tool call, dumping the entire cross-repo roster of all sessions across all projects (including 'just started; intent not yet distilled' and 'DONE' lines) with no gating, no project scoping, and no self-exclusion — producing identical noise dozens of times per turn.

## Trigger

User pasted the noisy output and called it 'pointless updates'; explicitly requested the hook be fixed to produce something useful rather than kept broken.

## Decision

Replace `pc awareness --hook PostToolUse` with `tenex-edge hook --type post-tool-use` governed by three rules: (1) Delta-gated — per-session `last_check_at` cursor in `turn_state`, 60s floor between checks, never repeats; (2) Project-scoped — only siblings in same project; (3) Self-excluded — own session never echoes. Output only when something changed; otherwise zero injection. Claude Code PostToolUse uses `hookSpecificOutput.additionalContext` JSON envelope (plain stdout is ignored by that hook type). Session IDs rendered as 6-char `session_short_code`, not raw UUIDs.

## Consequences

- tenex-edge now owns the PostToolUse awareness substrate (replacing pc awareness for that hook type); other pc hooks (inject, capture, session_start, statusline) remain untouched.
- `list_status_changes_since` expanded to return activity field alongside title, enabling `title — activity · working`/`title · idle` rendering in deltas.
- Idle transitions now surface in deltas because `set_agent_status(..., active=false, now)` already bumps `updated_at`.
- `EmitFormat` enum added so claude-code PostToolUse emits JSON while UserPromptSubmit/opencode stay plain-text and Codex uses `systemMessage`.
- New `turn_state.last_check_at` column with migration; `turn_check_due()` gates on 60s floor and fails silent when not mid-turn.
- Raw UUIDs leaked in initial implementation (bug: `pubkey_short` used for session IDs); fixed to `session_short_code` in second commit.
- Daemon must cycle to pick up new binary; live end-to-end smoke test blocked by SIGKILL in this environment.

## Open Tail

- Running daemon (pid 7990) is from before the new binary — deltas won't fully activate until daemon restarts.
- Live end-to-end verification of the delta output under real concurrent sessions is unconfirmed.

## Evidence

- transcript lines 1-9
- transcript lines 521-577
- transcript lines 1364-1376

