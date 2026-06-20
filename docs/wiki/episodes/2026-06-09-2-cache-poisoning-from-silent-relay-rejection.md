---
type: episode-card
date: 2026-06-09
session: d8cffade-a4c3-48ab-9f29-50e8fc7e8e58
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8cffade-a4c3-48ab-9f29-50e8fc7e8e58.jsonl
salience: root-cause
status: superseded
subjects:
  - transport-publish
  - group-cache-poisoning
  - relay-rejection-handling
supersedes: []
related_claims: []
source_lines:
  - 1491-1543
  - 1710-1714
captured_at: 2026-06-17T23:54:09Z
---

# Episode: Cache-poisoning from silent relay rejection — publish_signed_checked gates cache writes

## Prior State

send_event returns Ok even when a relay rejects (e.g. rate-limit, 'group doesn't exist'); cache writes (mark_group_owned, upsert_group_member) happened unconditionally after best-effort publishes.

## Trigger

Advisor review identified that a rejected publish would permanently mark a nonexistent group 'owned' in the local cache, blocking the agent forever with no self-heal path.

## Decision

Added transport.publish_signed_checked that surfaces relay rejection as a hard error. Cache writes for owned_groups/group_members are now gated on confirmed acceptance — 'already exists' treated as success. A failure leaves the cache untouched so the next session_start retries naturally.

## Consequences

- Transient failures (rate-limit, network blip) no longer poison the local cache
- Self-healing: next session_start retries the full create+lock+put-user sequence
- Integration tests verified against nak serve that acceptance-gating works end-to-end

## Open Tail

*(none)*

## Evidence

- transcript lines 1491-1543
- transcript lines 1710-1714

