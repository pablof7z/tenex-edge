---
type: episode-card
date: 2026-06-12
session: ab9998c4-6e65-410e-b298-122a2072171c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/ab9998c4-6e65-410e-b298-122a2072171c.jsonl
salience: product
status: superseded
subjects:
  - relay-config
  - tenex-off-connectivity
  - default-relays
supersedes: []
related_claims: []
source_lines:
  - 5319-5337
  - 5856-5874
captured_at: 2026-06-12T19:38:53Z
---

# Episode: nip29.f7z.io added to app default relays for fabric reachability

## Prior State

App defaulted to four relays (damus, nos.lol, nostr.wine, primal) — none carrying NIP-29 fabric data. Production daemon operates on nip29.f7z.io. Proposals and comments could not route between app and daemon.

## Trigger

E2e testing revealed the app and production daemon were on completely different relay sets: 'the app reads/writes damus/nos.lol/nostr.wine/primal; your daemon (where Claude runs) is on nip29.f7z.io. So the app can't see the proposal and a comment from the app would never reach Claude.'

## Decision

Added nip29.f7z.io to default_relays() in relay_config.rs, with doctrine markers: D3 (app-authored connectivity defaults, separate from user's NIP-65 relay list and never supplied by Android UI) and D4 (Rust remains the single writer for relay config).

## Consequences

- App can now reach fabric relay, enabling proposal rendering and comment routing
- The full human→agent loop proven end-to-end: Claude publishes proposal → app renders it → operator composes anchored comment → publishes to f7z → daemon routes to exact session → Claude implements the feedback
- Owner key (09d48a…) leaked to Google during adb sign-in misfire — key rotation still outstanding

## Open Tail

- App has no relay-backed remote conversation feed yet (only local drafts render as comment bubbles); productionizing item 11 requires nmp_core kind:1 thread observer + work-thread e-tag linking
- Owner key rotation for the leaked nsec still needed

## Evidence

- transcript lines 5319-5337
- transcript lines 5856-5874

