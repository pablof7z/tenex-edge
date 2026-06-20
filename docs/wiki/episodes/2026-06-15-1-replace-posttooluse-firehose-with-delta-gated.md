---
type: episode-card
date: 2026-06-15
session: a0037729-ad51-460a-880d-0a9699f6ee41
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a0037729-ad51-460a-880d-0a9699f6ee41.jsonl
salience: product
status: active
subjects:
  - post-tool-use-awareness
  - turn-state-cursor
  - hook-output-format
supersedes: []
related_claims: []
source_lines:
  - 1-51
  - 521-587
  - 629-680
  - 1193-1211
  - 1377-1464
captured_at: 2026-06-18T00:35:40Z
---

# Episode: Replace PostToolUse firehose with delta-gated sibling awareness

## Prior State

The `pc awareness` PostToolUse hook fired on every tool call and dumped the entire cross-repo roster — global, ungated, no debounce, repeated identically dozens of times per turn. Agents saw pointless noise like `just started; intent not yet distilled` and `DONE master: finished` from all projects regardless of relevance.

## Trigger

User identified the PostToolUse output as 'pointless updates' and directed that the hook produce something useful rather than remain broken or disabled.

## Decision

Replaced `pc awareness --hook PostToolUse` with `tenex-edge hook --type post-tool-use`. The new hook is delta-gated (per-session `last_check_at` cursor; same fact never shown twice; no output when nothing changed), project-scoped (only sessions in this project), self-excluded (agent never sees its own status), and 60s debounced (`turn_check_due()` returns `None` if <60s since last check; first check of a turn always fires). Claude Code PostToolUse emits via `hookSpecificOutput.additionalContext` JSON envelope (plain stdout is silently ignored by Claude Code). Session IDs render as 6-char hash short codes matching `who`/tmux.

## Consequences

- New `last_check_at` column in `turn_state` table (guarded ALTER TABLE migration; also added to CREATE TABLE for in-memory DBs)
- `list_status_changes_since` now returns the activity field alongside title/project/slug, so deltas show `title — activity · working` or `title · idle`
- Session IDs initially rendered as raw UUIDs (copy-paste-unusable for `send --to`); fixed to canonical `session_short_code` in a follow-up commit
- `EmitFormat` enum added to distinguish output envelopes across harness types (claude-code PostToolUse → JSON; others → plain/systemMessage)
- `pc awareness` PostToolUse removed from `~/.claude/settings.json`; timeout set to 10s matching per-tool-call cadence
- Daemon must cycle to a new binary for the delta logic to work live; the running daemon from a prior build served old code until restart

## Open Tail

- Daemon restart was deferred (outward-facing; briefly affects all live sessions); delta half won't fire until daemon cycles to the new binary

## Evidence

- transcript lines 1-51
- transcript lines 521-587
- transcript lines 629-680
- transcript lines 1193-1211
- transcript lines 1377-1464

