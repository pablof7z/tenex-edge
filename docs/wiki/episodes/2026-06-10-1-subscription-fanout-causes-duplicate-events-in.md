---
type: episode-card
date: 2026-06-10
session: 56f9fe89-5ff7-4e5b-b202-334cd7629d42
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/56f9fe89-5ff7-4e5b-b202-334cd7629d42.jsonl
salience: root-cause
status: superseded
subjects:
  - daemon-subscription-dedup
  - handle-incoming
supersedes: []
related_claims: []
source_lines:
  - 78-421
captured_at: 2026-06-18T00:06:08Z
---

# Episode: Subscription fanout causes duplicate events in tail

## Prior State

The daemon opened one REQ subscription per (hosted_agent × project). Relay delivers the same event once per matching subscription ID, so handle_incoming was called N times for a single published event (e.g. 15× with 5 agents × 3 projects).

## Trigger

User observed a single sent message printed 15 times in `tenex tail` output (lines 78–95).

## Decision

Added a 512-slot `seen_events` VecDeque ring buffer to `DaemonState`; `handle_incoming` now short-circuits on any event ID already in the ring, deduplicating across all subscription fanout.

## Consequences

- Each inbound event is processed exactly once regardless of how many overlapping subscriptions match it.
- The ring buffer is unbounded at 512 entries; extremely high throughput could theoretically cycle old IDs out, but in practice the window is generous.
- The underlying structural issue (N subscriptions per N agents × M projects) remains; only the symptom is deduplicated.

## Open Tail

- Consider consolidating overlapping subscriptions into a single union REQ to reduce relay load and avoid the fanout entirely.

## Evidence

- transcript lines 78-421

