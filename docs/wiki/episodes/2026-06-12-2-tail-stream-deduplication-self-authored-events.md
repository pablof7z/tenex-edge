---
type: episode-card
date: 2026-06-12
session: 0bc06206-1f30-4e35-8373-f31d0f5c1dcc
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/0bc06206-1f30-4e35-8373-f31d0f5c1dcc.jsonl
salience: product
status: active
subjects:
  - tail-stream-dedup
  - thread-attribution
supersedes:
  - 2026-06-10-1-subscription-fanout-causes-duplicate-events-in
related_claims: []
source_lines:
  - 4823-4848
  - 4907-5057
  - 5060-5063
  - 5142-5182
  - 5252-5255
captured_at: 2026-06-18T00:13:21Z
---

# Episode: Tail stream deduplication: self-authored events suppressed, canonical thread attribution

## Prior State

Tail stream showed ~12 duplicate lines per message (empty sender slug, echo/outbound double-count). Thread attribution used a heuristic lookup (latest_thread_for_inbound) that could misattribute.

## Trigger

Live e2e testing of the merged fabric branch revealed duplicate tail lines and wrong thread grouping. Initial fix of gating all of handle_incoming broke startup catch-up (subscription replay is load-bearing).

## Decision

Self-authored events never derive a tail line — deterministic suppression instead of a race-prone echo filter. Materialization always runs (catch-up depends on it), but only tail emission is first-sight-gated. Heuristic thread resolution deleted; canonical thread ID from the materializer is now the sole source of thread attribution.

## Consequences

- Exactly one tail line per message with correct slug resolution
- Startup catch-up continues to work because materialization is ungated
- Thread naming uses materializer's canonical thread ID (e.g. #thr-18b8) instead of a store heuristic

## Open Tail

*(none)*

## Evidence

- transcript lines 4823-4848
- transcript lines 4907-5057
- transcript lines 5060-5063
- transcript lines 5142-5182
- transcript lines 5252-5255

