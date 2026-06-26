---
type: episode-card
date: 2026-06-26
session: fb3a3db1-26e3-4a15-9745-056690b09083
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/fb3a3db1-26e3-4a15-9745-056690b09083.jsonl
salience: root-cause
status: active
subjects:
  - tmux-statusline
  - session-identification
supersedes: []
related_claims: []
source_lines:
  - 208-226
  - 319-338
captured_at: 2026-06-26T08:56:37Z
---

# Episode: Format variable quoting mismatch causes silent statusline failure when session identifier is unset

## Prior State

The statusline feature implemented session awareness via tmux user variables (@te_session) passed to the statusline command; code comments document using quoted form #{q:@te_session}

## Trigger

User reports status bar displays placeholder text instead of session info; investigation reveals the statusline command returns empty when @te_session is unset

## Decision

Root cause identified: implementation uses unquoted #{@te_session} instead of documented quoted #{q:@te_session}; in tmux, unquoted format variables expand to nothing (not empty string) when undefined, causing the shell to omit the --session argument entirely, which clap rejects with exit code 2 and produces no output

## Consequences

- Status bar silently shows empty content for any session without @te_session set (all pre-feature sessions remain broken)
- Silent failure mode obscures root cause—no stderr or visible error indication in tmux status bar context
- Code documentation contradicts implementation (comments say #{q:@te_session}, implementation uses #{@te_session})
- Fix requires both quoting correction and likely fallback output to prevent empty/broken status bars

## Open Tail

- Implementation: change format string to use #{q:@te_session}
- Evaluate whether statusline command should emit fallback output when session data unavailable

## Evidence

- transcript lines 208-226
- transcript lines 319-338

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-1-format-variable-quoting-mismatch-causes-silent.json`](transcripts/2026-06-26-1-format-variable-quoting-mismatch-causes-silent.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-1-format-variable-quoting-mismatch-causes-silent.json`](transcripts/raw/2026-06-26-1-format-variable-quoting-mismatch-causes-silent.json)
