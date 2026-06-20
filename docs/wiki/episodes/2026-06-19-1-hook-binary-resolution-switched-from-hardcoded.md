---
type: episode-card
date: 2026-06-19
session: 8fbcc279-f528-4fb3-a2f8-2aec4e9c25aa
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8fbcc279-f528-4fb3-a2f8-2aec4e9c25aa.jsonl
salience: architecture
status: active
subjects:
  - hook-binary-resolution
  - claude-settings
supersedes: []
related_claims: []
source_lines:
  - 1-72
captured_at: 2026-06-19T11:54:45Z
---

# Episode: Hook binary resolution switched from hardcoded path to PATH lookup

## Prior State

Claude Code hooks hardcoded the absolute path to the tenex-edge binary (/Users/pablofernandez/src/tenex-edge/target/debug/tenex-edge), pinning to a specific local dev build location.

## Trigger

User directive: 'change claude/codex binary to use the one in $PATH, not /Users/pablofernandez/src/tenex-edge/target/debug/tenex-edge'

## Decision

All 4 hook commands changed from absolute dev paths to bare tenex-edge, relying on PATH resolution — matching the pattern already used by the session-end hook.

## Consequences

- Hooks are now portable across machines and users instead of pinned to one developer's build directory
- System now depends on tenex-edge being installed and resolvable via PATH
- Decouples settings.json from specific local build artifacts

## Open Tail

- Demo scripts in worktree directories still reference $ROOT/target/debug/tenex-edge — may need similar treatment

## Evidence

- transcript lines 1-72

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-19-1-hook-binary-resolution-switched-from-hardcoded.json`](transcripts/2026-06-19-1-hook-binary-resolution-switched-from-hardcoded.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-19-1-hook-binary-resolution-switched-from-hardcoded.json`](transcripts/raw/2026-06-19-1-hook-binary-resolution-switched-from-hardcoded.json)
