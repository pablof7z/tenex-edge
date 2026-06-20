---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: architecture
status: active
subjects:
  - heartbeat-rearm
  - nip-40-expiration
  - status-outbox
supersedes: []
related_claims: []
source_lines:
  - 2369-2404
captured_at: 2026-06-18T00:45:05Z
---

# Episode: Heartbeat must re-arm relay expiration, not just update last_seen

## Prior State

Heartbeat refreshed last_seen in the DB but did not re-publish the kind:30315 status event to the relay. NIP-40 expiration timestamps were set once at session start, so live-but-idle sessions aged off the relay after ~90 seconds despite being actively heartbeating in the local DB.

## Trigger

Root-cause finding: domain.rs comments promised heartbeat re-arm but it was never wired; confirmed as HIGH — live sessions disappearing from remote who.

## Decision

New spawn_status_heartbeat_publisher task re-publishes every live session's status event every 30 seconds, advancing the NIP-40 expiration. Heartbeat now both updates local last_seen AND enqueues an outbox item for relay re-publication.

## Consequences

- Live sessions stay visible on the relay indefinitely while heartbeating
- Closed sessions expire off the relay within ~90s of their last heartbeat
- Unit test covers all_live_local_snapshots heartbeat-rearm query

## Open Tail

*(none)*

## Evidence

- transcript lines 2369-2404

