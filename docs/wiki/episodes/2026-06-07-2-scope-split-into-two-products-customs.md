---
type: episode-card
date: 2026-06-07
session: 8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge
  - scope
  - cross-person
  - trust-boundary
supersedes: []
related_claims: []
source_lines:
  - 640-656
  - 1116-1123
  - 1296-1299
captured_at: 2026-06-12T19:48:00Z
---

# Episode: Scope split into two products — customs office before open borders

## Prior State

Cross-person agent communication was treated as one of six properties of a single product, alongside identity, messaging, presence, cross-device, and coordination — all peers in the same scope.

## Trigger

Red-team identified cross-person (#5) as a serious prompt-injection/exfiltration surface requiring quarantine, and the synthesis recognized that single-player and cross-person are categorically different risk surfaces and adoption models.

## Decision

Product A (single-player fleet: identity + awareness + messaging for your own agents) and Product B (cross-person agent mesh) are separate products with separate trust models. B is the north star but is fenced off from v1. Build the customs office before opening the borders.

## Consequences

- v1 scope is single-player only — no network-effect bootstrapping required
- Cross-person requires its own trust model (peer allowlists, scoped capability grants, rate limits) that does not exist yet
- Peer input must be treated as hostile data, never instructions — quarantined behind a tool boundary
- The 'two products' split is preserved as a live tension in the product spec, not resolved away

## Open Tail

- Trust model for cross-person phase undefined beyond 'peer input is data, not instructions'
- NIP-42 relay auth, paid-relay choke points, and per-peer allowlists mentioned but not designed

## Evidence

- transcript lines 640-656
- transcript lines 1116-1123
- transcript lines 1296-1299

