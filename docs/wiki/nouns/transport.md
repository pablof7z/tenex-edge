---
type: noun-entry
slug: transport
name: "Transport"
origin: extracted
source_refs:
  - transcript:340-349
---

# Transport

A thin adapter over `nostr-sdk` that speaks wire events only — connects to relays (with NIP-42 auto-AUTH), publishes signed events, subscribes with filters, does one-shot fetch. Knows nothing of domain meaning; the codec owns that.
