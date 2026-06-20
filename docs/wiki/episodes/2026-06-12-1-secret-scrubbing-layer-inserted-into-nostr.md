---
type: episode-card
date: 2026-06-12
session: 1f333238-0710-47f2-bae9-9d5f54b09634
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1f333238-0710-47f2-bae9-9d5f54b09634.jsonl
salience: product
status: active
subjects:
  - secret-redaction
  - transport-publish
  - nostr-event-privacy
supersedes: []
related_claims: []
source_lines:
  - 1-426
captured_at: 2026-06-18T00:09:32Z
---

# Episode: Secret-scrubbing layer inserted into Nostr event publishing

## Prior State

Events published to Nostr relays carried raw user/agent content with no secret redaction; any credential embedded in prompts, status lines, or DMs would be broadcast in cleartext.

## Trigger

User directive to avoid leaking secrets in published events, initially suggesting keyhog; investigation revealed keyhog/secretscan are file-oriented scanners, not runtime &str → String redaction libraries.

## Decision

Build a regex-based `scrub_secrets` function inside `Transport`, intercepting between `builder.build()` and `keys.sign_event()` in both `publish_signed` and `publish_signed_checked`. After scrubbing `unsigned.content`, reset `unsigned.id = None` so nostr-sdk recomputes the event ID over the scrubbed content. Added `regex = "1"` as the only new dependency. Pattern set covers AWS, GitHub, Slack, Google, Anthropic, OpenAI, Nostr nsec, PEM private keys, and (after user feedback) Ollama `<32-hex>.<alphanum>` keys.

## Consequences

- All user-content paths (prompts, status, DMs, proposals) funnel through the two publish methods and are now scrubbed before signing.
- NIP-29 admin events (kinds 9000/9002/9007) have empty content, so scrubbing is a zero-cost no-op for them.
- Event ID is computed post-scrub, guaranteeing cryptographic consistency — without the `id = None` reset, `sign_event` errors with `InvalidId`.
- Generic pattern shapes (e.g. Ollama's `<32-hex>.<alphanum>`) risk false positives; targeted prefix-based patterns are preferred where possible.

## Open Tail

- New secret formats may require additional regex patterns over time.
- The scrubber only covers `unsigned.content`; tags or other fields are not yet inspected.

## Evidence

- transcript lines 1-426

