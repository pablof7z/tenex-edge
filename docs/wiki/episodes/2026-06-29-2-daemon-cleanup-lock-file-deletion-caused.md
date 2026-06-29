---
type: episode-card
date: 2026-06-29
session: b07a57a3-67a1-4c44-a8fc-58a1bb97860a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b07a57a3-67a1-4c44-a8fc-58a1bb97860a.jsonl
salience: root-cause
status: active
subjects:
  - daemon-startup-lock
  - cleanup-lock-file-race
  - state-db-contention
supersedes: []
related_claims: []
source_lines:
  - 817-855
  - 839-848
captured_at: 2026-06-29T10:36:07Z
---

# Episode: Daemon cleanup() lock-file deletion caused two-daemon race on state.db

## Prior State

cleanup() deleted the lock file (along with the socket) when the daemon shut down. When a new daemon started while the old one was still exiting, cleanup() removed the lock file inode, the new daemon acquired a fresh flock on a new file, and both daemons contended for state.db — causing the client to hang indefinitely waiting for RPC responses.

## Trigger

User reported tenex-edge channels create hanging forever with 'waiting for daemon to answer RPCs'. Investigation showed the daemon was bound to the socket (PID 79353) but handshake probes failed, and the daemon log showed repeated relay-not-connected errors from a stale daemon process.

## Decision

cleanup() no longer removes the lock file. The flock persists on the same inode until the old daemon's process actually exits, so try_acquire() on the new lock file correctly blocks until the old daemon releases it.

## Consequences

- Two daemons can no longer race on state.db — the flock serialization now works correctly across process lifecycle boundaries
- Stale lock files from crashed daemons are handled by flock semantics (released when the process dies) rather than file deletion

## Open Tail

*(none)*

## Evidence

- transcript lines 817-855
- transcript lines 839-848

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-2-daemon-cleanup-lock-file-deletion-caused.json`](transcripts/2026-06-29-2-daemon-cleanup-lock-file-deletion-caused.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-2-daemon-cleanup-lock-file-deletion-caused.json`](transcripts/raw/2026-06-29-2-daemon-cleanup-lock-file-deletion-caused.json)
