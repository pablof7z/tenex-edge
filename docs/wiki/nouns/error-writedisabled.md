---
type: noun-entry
slug: error-writedisabled
name: "Error::WriteDisabled"
origin: extracted
source_refs:
  - transcript:587-594
---

# Error::WriteDisabled

A client-side error from nostr-relay-pool 0.44.1 whose Display text is 'write actions are disabled'. It fires when the daemon's in-memory Relay object has RelayServiceFlags missing WRITE, causing the SDK to refuse to put EVENT messages on the wire — the relay never sees or rejects the publish.
