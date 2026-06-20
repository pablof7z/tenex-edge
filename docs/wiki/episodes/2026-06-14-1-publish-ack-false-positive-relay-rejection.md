---
type: episode-card
date: 2026-06-14
session: d683a556-03b8-4827-b84d-5395cd3610af
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d683a556-03b8-4827-b84d-5395cd3610af.jsonl
salience: root-cause
status: active
subjects:
  - publish-ack-false-positive
  - relay-rejection-handling
  - doctor-probe-honesty
supersedes:
  - 2026-06-09-2-cache-poisoning-from-silent-relay-rejection
related_claims: []
source_lines:
  - 5-17
  - 121-133
  - 189-233
  - 487-497
  - 499-530
  - 531-596
  - 835-855
captured_at: 2026-06-18T00:22:02Z
---

# Episode: Publish-ack false positive: relay rejection surfaced as success

## Prior State

nostr-sdk's send_event/send_event_builder resolve Ok on transmission; the real per-relay NIP-01 OK/FAIL verdict lives in output.success/output.failed. The propose and doctor codepaths only consumed the optimistic write-side ack, so a NIP-29 relay rejecting the event (e.g. blocked: unknown member) still reported 'publish: OK' / 'published proposal …' with exit 0 — silent data loss.

## Trigger

Issue #1: tenex-edge propose consistently reports 'published proposal' with exit 0, but kind:30023 events never become retrievable. Doctor confirmed: publish OK, read-back returns 0 events.

## Decision

Added assert_relay_accepted() helper that fails unless ≥1 relay is in output.success (surfacing relay's stated rejection reason). publish_signed_checked now returns EventId and routes through the helper. provider.publish_checked() makes relay rejection a hard error for propose. provider.is_retrievable() does a post-publish read-back. CLI warns loudly (yellow 'warning:') when relay ACKed but event isn't retrievable. doctor_probe uses checked publish so its publish: line reflects the true relay verdict.

## Consequences

- Relay rejections now surface as hard errors (CLI exits nonzero with relay's reason) instead of false-positive success
- Doctor's publish: line reports honest relay verdicts instead of always showing OK
- Internal read-back after propose catches relay ACK-then-drop scenarios
- publish_signed_checked return type changed from () to EventId — all callers updated
- Multiple relays already structurally supported (Config.relays is Vec<String>); no code change needed for fallback

## Open Tail

- Running daemon still has old binary; restart needed for fix to take effect
- Concurrent agent committed parts of this fix under an unrelated commit message

## Evidence

- transcript lines 5-17
- transcript lines 121-133
- transcript lines 189-233
- transcript lines 487-497
- transcript lines 499-530
- transcript lines 531-596
- transcript lines 835-855

