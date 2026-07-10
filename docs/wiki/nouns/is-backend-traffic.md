---
type: noun-entry
slug: is-backend-traffic
name: "is_backend_traffic"
origin: extracted
source_refs:
  - transcript:132-133
---

# is_backend_traffic

A filter that excludes any chat event whose author OR any p-tag recipient is either the daemon's own backend_pubkey (mgmt key) or a pubkey whose cached kind:0 sets is_backend. Exists only in the fabric_context (hook/awareness) path, not in channel read.
