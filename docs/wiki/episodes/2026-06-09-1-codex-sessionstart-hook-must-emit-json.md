---
type: episode-card
date: 2026-06-09
session: 2cee1bc6-0f1a-4746-9de6-68ca1a7e2737
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/2cee1bc6-0f1a-4746-9de6-68ca1a7e2737.jsonl
salience: root-cause
status: active
subjects:
  - codex-hook-output-format
  - session-start-hook
supersedes: []
related_claims: []
source_lines:
  - 1-54
  - 109-204
  - 504-515
captured_at: 2026-06-17T23:44:19Z
---

# Episode: Codex SessionStart hook must emit JSON, not plain text

## Prior State

te-hook.py printed plain text to stdout for session-start; Codex rejected it with 'hook returned invalid session start JSON output'

## Trigger

Codex SessionStart hook parses stdout as JSON with schema {systemMessage, suppressOutput, stopReason, hookSpecificOutput} — discovered by extracting embedded JSON Schema from the Codex binary

## Decision

All Codex hook output must be JSON; session-start now emits json.dumps({"systemMessage": msg}). The same schema applies to all Codex hook types (UserPromptSubmit, PostToolUse, Stop).

## Consequences

- All Codex hook types share the same output schema; systemMessage is the injection vector for agent-visible context
- The --json flag on turn-start/turn-check wraps output in {"systemMessage": ...} for Codex; plain text for Claude Code
- Empty/no-op hooks must still emit valid JSON or nothing (not arbitrary text)

## Open Tail

*(none)*

## Evidence

- transcript lines 1-54
- transcript lines 109-204
- transcript lines 504-515

