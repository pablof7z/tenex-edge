---
type: episode-card
date: 2026-06-15
session: a0037729-ad51-460a-880d-0a9699f6ee41
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a0037729-ad51-460a-880d-0a9699f6ee41.jsonl
salience: product
status: superseded
subjects:
  - post-tool-use-hook
  - tenex-edge-awareness
  - sibling-session-delta
supersedes: []
related_claims: []
source_lines:
  - 1-52
  - 235-242
  - 278-280
  - 521-584
  - 586-628
  - 1193-1211
captured_at: 2026-06-15T09:13:20Z
---

# Episode: Replace PostToolUse firehose with delta-gated project-scoped awareness

## Prior State

PostToolUse hook ran `pc awareness --hook PostToolUse` on every tool call, dumping the full cross-repo roster (all projects, all sessions) ungated — including 'just started; intent not yet distilled' and 'DONE' lines. Identical output repeated dozens of times per turn. Claude Code PostToolUse was considered unwirable for tenex-edge because the JSON output contract was unverified.

## Trigger

User complained the hook produced 'pointless updates' showing the entire peer roster; when assistant proposed removing it, user demanded it be fixed to produce useful output — 'fucking fix the hook to produce something useful, not just keep it broken and disabled!' — specifying delta-gated session titles scoped to the project.

## Decision

Replace `pc awareness` with `tenex-edge hook --host claude-code --type post-tool-use`. Three noise-killing rules: (1) Delta-gated — per-session cursor `turn_state.last_check_at` advances on each check; same fact never shown twice; zero output when nothing changed. (2) Project-scoped — only sibling sessions in this project. (3) Self-excluded — own session never echoes back. 60s debounce floor (first check of a turn always fires). Output uses Claude Code's required JSON envelope `{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"..."}}` (plain stdout is silently discarded by Claude Code).

## Consequences

- New `last_check_at` column in `turn_state` table; `turn_check_due()` gates emission to ≥60s since last check and only during an active turn
- `list_status_changes_since` now returns the activity field alongside title, enabling 'title — activity' rendering in deltas
- Idle transitions (falling-edge → `active=false`) already bump `updated_at`, so deltas now surface 'title · idle' entries
- New `EmitFormat` enum: `PostToolUseJson` for Claude Code PostToolUse, `JsonSystemMessage` for Codex, `PlainText` for UserPromptSubmit/opencode — prevents silent output loss
- Running daemon must cycle (restart) to serve the new `rpc_turn_check` logic; until then the hook returns old inbox-peek-only output
- Other `pc` hooks (inject, capture, session_start, statusline) remain untouched

## Open Tail

- Daemon restart needed before delta-gated output fires live; current daemon (pid 7990, started Jun 14) predates the new binary
- End-to-end live smoke test unconfirmed — CLI calls to daemon got SIGKILLed in this session environment

## Evidence

- transcript lines 1-52
- transcript lines 235-242
- transcript lines 278-280
- transcript lines 521-584
- transcript lines 586-628
- transcript lines 1193-1211

