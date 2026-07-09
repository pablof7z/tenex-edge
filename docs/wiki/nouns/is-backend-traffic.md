---
type: noun-entry
slug: is-backend-traffic
name: "is_backend_traffic"
origin: extracted
source_refs:
  - transcript:132-132
---

# is_backend_traffic

A filter in fabric_context that excludes any chat event whose author OR any p-tag recipient is either the daemon's own backend_pubkey (mgmt key) or a pubkey whose cached kind:0 profile sets is_backend.
