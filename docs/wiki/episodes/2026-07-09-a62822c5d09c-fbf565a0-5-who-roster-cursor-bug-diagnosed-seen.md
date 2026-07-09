---
type: episode-card
date: 2026-07-09
session: a62822c5-d09c-4a83-9251-a3856d276ac4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a62822c5-d09c-4a83-9251-a3856d276ac4.jsonl
salience: root-cause
status: active
subjects:
  - who-command
  - roster-display
  - seen-cursor-gating
supersedes: []
related_claims: []
source_lines:
  - 1234-1260
  - 1932-1940
captured_at: 2026-07-09T20:52:45Z
---

# Episode: `who` roster-cursor bug diagnosed: seen_cursor gates agent roster display

## Prior State

`tenex-edge who` was expected to display all configured agents (14 on this machine). It was only showing 2.

## Trigger

User noticed `who` listing only 2 agents when 14 are configured on disk (lines 1234–1260).

## Decision

Diagnosis only (no fix applied): `who` applies the session's chat-delta `seen_cursor` as a gate on the agent roster, so a several-turns-deep session only sees roster rows republished after that cursor (2 of 14), while a fresh session/hook injection at cursor 0 sees all 14.

## Consequences

- Root cause identified as cursor-gating on the roster query, not an agent-configuration or profile-loading issue
- Future fix must decouple roster display from the chat-delta seen_cursor, or reset/skip the cursor for roster reads

## Open Tail

- Fix not yet implemented — deferred until git consolidation with concurrent agent settles and daemon restart is scheduled

## Evidence

- transcript lines 1234-1260
- transcript lines 1932-1940

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-a62822c5d09c-fbf565a0-5-who-roster-cursor-bug-diagnosed-seen.json`](transcripts/2026-07-09-a62822c5d09c-fbf565a0-5-who-roster-cursor-bug-diagnosed-seen.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-5-who-roster-cursor-bug-diagnosed-seen.json`](transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-5-who-roster-cursor-bug-diagnosed-seen.json)
