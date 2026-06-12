---
type: episode-card
date: 2026-06-09
session: 2cee1bc6-0f1a-4746-9de6-68ca1a7e2737
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/2cee1bc6-0f1a-4746-9de6-68ca1a7e2737.jsonl
salience: root-cause
status: active
subjects:
  - codex-hook-session-start
  - hook-output-format
supersedes: []
related_claims: []
source_lines:
  - 1-54
  - 826-864
captured_at: 2026-06-12T19:57:01Z
---

# Episode: Codex SessionStart hook must output JSON, not plain text

## Prior State

The Codex te-hook.py script printed a plain-text string to stdout on successful session-start, which Codex rejected with 'hook returned invalid session start JSON output'

## Trigger

User reported the SessionStart hook failure on Codex; root-cause found by extracting the embedded JSON Schema from the Codex binary — all hook types require JSON output with fields {systemMessage, suppressOutput, stopReason, hookSpecificOutput}

## Decision

Wrap session-start output as json.dumps({"systemMessage": msg}) instead of plain-text print(); this schema applies to ALL Codex hook types (SessionStart, UserPromptSubmit, PostToolUse, Stop)

## Consequences

- All Codex hook output must be JSON with systemMessage field — plain text is invalid
- The wait-for-mention hint is now delivered via systemMessage injection rather than raw stdout
- Future hook types (PostToolUse) inherit the same output contract

## Open Tail

*(none)*

## Evidence

- transcript lines 1-54
- transcript lines 826-864

