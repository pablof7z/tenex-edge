---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: architecture
status: active
subjects:
  - relay-liveness
  - kind-30315
  - session-status-event
supersedes: []
related_claims: []
source_lines:
  - 1279-1313
captured_at: 2026-06-16T11:28:20Z
---

# Episode: Relay liveness model: expiry tags on heartbeat events, not tombstones or freshness-only

## Prior State

Two proposals were on the table: Codex recommended active tombstones (publish lifecycle:ended/superseded terminal events); Opus recommended stable identity + freshness windows (readers check last_seen, don't read liveness from relay). The existing design published kind:30315 as never-expiring heartbeat with no liveness signal beyond recency of created_at.

## Trigger

User explicitly rejected both proposals and directed: 'just include an expiry tag — that's why we publish it as a heartbeat, so we know when the session stopped being active by the lack of an update.'

## Decision

Use expiry tags on heartbeat events. Liveness is determined by whether an update arrives before the expiry window elapses — absence of update past expiry means the session is no longer active. No separate tombstone events needed.

## Consequences

- No extra relay writes on session end/supersede — the heartbeat's expiry tag communicates liveness
- Readers determine 'is it alive?' by comparing current time against the expiry window indicated in the event
- Old events simply expire rather than requiring active cleanup
- Consistent with the existing heartbeat publish pattern
- Stable session_key (from the aggregate architecture) removes most orphan d-tags; expiry handles the rest

## Open Tail

- Exact expiry window duration not yet specified
- NIP-40 expiration tag vs custom expiry field decision pending

## Evidence

- transcript lines 1279-1313

