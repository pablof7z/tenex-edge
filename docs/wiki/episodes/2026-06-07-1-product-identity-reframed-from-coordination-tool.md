---
type: episode-card
date: 2026-06-07
session: 8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666.jsonl
salience: reversal
status: active
subjects:
  - tenex-edge
  - product-identity
  - coordination
supersedes: []
related_claims: []
source_lines:
  - 640-683
  - 1087-1142
  - 1167-1194
captured_at: 2026-06-12T19:48:00Z
---

# Episode: Product identity reframed from coordination tool to agent citizenship protocol

## Prior State

The product agent positioned tenex-edge as a coordination tool for dev fleets, with advisory locking (collision avoidance) and shared bug dedup as the spine. The exciting feature was agents coordinating work across repos.

## Trigger

Red-team critique demonstrated that coordination/locking is the least load-bearing and most redundant-with-git feature, and proposed a cheap collision-counting experiment to falsify the premise. Simultaneously, the user's todo/podcast example revealed a bigger abstraction — agents dissolving the human-as-glue across all apps, not just dev fleets.

## Decision

Tenex-edge is an agent citizenship protocol: durable cross-host identity + presence that outlives any tool, not a coordination tool. Coordination is demoted from founding pillar to testable hypothesis (run collision-frequency experiment before investing). The defensible core is vendor-independent agent identity and provenance, not advisory locks.

## Consequences

- Product scope widens beyond dev fleets to all apps in a user's life
- Coordination (locks/dedup) becomes an experiment, not the load-bearing feature
- Vendor-independent identity is the center of gravity — survives even if a host absorbs coordination features
- One-liner reframed to: 'Citizenship for your agents — a durable identity and a shared world, no matter which tool they're running in'
- The 'plugin is the straw; the fabric is the milkshake' — distribution via hosts, durable asset is the identity/fabric layer

## Open Tail

- Collision-frequency experiment not yet run — if collisions are frequent, advisory awareness (not authority) could still be promoted
- Research into SOTA and X/Twitter commentary still in flight to validate demand signal

## Evidence

- transcript lines 640-683
- transcript lines 1087-1142
- transcript lines 1167-1194

