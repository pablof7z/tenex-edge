---
type: episode-card
date: 2026-06-15
session: a0037729-ad51-460a-880d-0a9699f6ee41
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a0037729-ad51-460a-880d-0a9699f6ee41.jsonl
salience: product
status: active
subjects:
  - post-tool-use-awareness
  - sibling-delta-injection
  - session-status-cursor
supersedes:
  - 2026-06-15-1-replace-posttooluse-awareness-firehose-with-delta
related_claims: []
source_lines:
  - 1-51
  - 519-524
  - 525-566
  - 569-587
  - 629-629
  - 687-687
  - 1193-1207
  - 1364-1377
  - 1431-1442
captured_at: 2026-06-15T11:21:39Z
---

# Episode: Replace pc-awareness PostToolUse firehose with delta-gated tenex-edge awareness

## Prior State

`pc awareness --hook PostToolUse` fired on every tool call, dumping the entire global cross-repo roster — all sessions across all projects (nmp, podcast-player, chirp, etc.), including stale lines like 'just started; intent not yet distilled' and 'finished'. It was global, ungated, and repeated identically dozens of times per turn. Agents saw noise, not signal.

## Trigger

User pasted the firehose output and flagged it as 'pointless updates' that are 'incredibly noisy and pointless', demanding it be fixed to produce something useful.

## Decision

Replace `pc awareness` on PostToolUse with tenex-edge `hook --type post-tool-use`: a delta-gated, project-scoped, self-excluded, 60s-debounced system. It emits only when something actually changed in the same project, never about the agent's own session, at most once per 60s. Uses Claude Code's `hookSpecificOutput.additionalContext` JSON envelope (plain stdout is silently discarded by PostToolUse — this is why the feature was previously unwired). Direct inbox messages remain immediate and unrate-limited.

## Consequences

- New `turn_state.last_check_at` cursor column with `turn_check_due()` gating — first check of a turn always fires, subsequent checks gated to ≥60s; cursor writes happen inside the daemon (single-writer, no multiwriter risk).
- New `EmitFormat` enum routes output through the correct envelope per harness: `AdditionalContext` JSON for Claude Code PostToolUse, plain text for UserPromptSubmit/open-code, `systemMessage` for Codex.
- `list_status_changes_since` extended to return the activity field alongside title — deltas now render 'title — activity · working' or 'title · idle'.
- Session IDs rendered as 6-char short codes (via `session_short_code`), not raw UUIDs — matched `who`/tmux display so they're copy-pasteable into `send --to`. Initial live deployment leaked full UUIDs; fixed in a follow-up commit.
- `pc awareness` removed from PostToolUse hooks in settings.json; other `pc` hooks (inject, capture, session_start, statusline) left untouched.
- 189 unit tests pass including new tests for the 60s gate, cursor advance, self-exclusion, activity rendering, idle transitions, and short-code format.

## Open Tail

- Live daemon version-skew: the long-running daemon must cycle to pick up the new binary for the delta logic to fully take effect.
- The 60s debounce floor was set mid-implementation (initially 30s, bumped to 60s); may need tuning based on real-world chattiness.
- PostToolUse hook timeout set to 10s (matching `pc awareness` precedent); actual daemon RPC is sub-millisecond but pathological daemon hangs could block tool calls.

## Evidence

- transcript lines 1-51
- transcript lines 519-524
- transcript lines 525-566
- transcript lines 569-587
- transcript lines 629-629
- transcript lines 687-687
- transcript lines 1193-1207
- transcript lines 1364-1377
- transcript lines 1431-1442

