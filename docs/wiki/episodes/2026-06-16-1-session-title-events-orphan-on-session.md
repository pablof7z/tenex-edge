---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: root-cause
status: active
subjects:
  - session-title-keying
  - kind30315-lifecycle
supersedes: []
related_claims: []
source_lines:
  - 229-370
captured_at: 2026-06-16T10:51:31Z
---

# Episode: Session-title events orphan on session-id rotation — root cause identified

## Prior State

Session titles are stored per session_id (cur_title → session_status.text → ["title"] tag on kind:30315) and the events are never expired; each session_id gets its own replaceable event keyed by d="<project>:<session_id>". The implicit assumption was that one session_id maps to one conversation.

## Trigger

Investigation into 'two competing titles' revealed that harnesses (claude-code, codex) rotate session_id on resume/clear/compaction, spawning a new session_id for the same logical conversation. The old session's cancel_session only signals the task — it does NOT delete or expire the old kind:30315 event, and the commit history (5e7a34d1) explicitly made these events non-expiring.

## Decision

Root cause diagnosed: title events are keyed per session_id rather than per conversation, and there is no tombstone/expiration mechanism for superseded sessions' 30315 events. A stale sibling killed by cancel_session leaves its titled event on the relay forever, producing competing titles for the same conversation.

## Consequences

- Multiple kind:30315 events with different titles can coexist for the same logical conversation on the relay
- Old titled events are never cleaned up — they persist indefinitely because NIP-40 expiration was deliberately omitted
- The who-list shows separate entries per session_id (e.g. 5144b7, 3d3439, 7f1881) rather than per conversation
- Any fix requires either: (a) publishing a deletion/expiration event for the superseded session's 30315 when a stale sibling is killed, or (b) re-keying the title event by something stable across session rotations

## Open Tail

- No decision yet on whether to delete/expire superseded session's 30315 on kill, or re-key the event by a conversation-stable identifier

## Evidence

- transcript lines 229-370

