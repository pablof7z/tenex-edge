---
type: noun-entry
slug: signerreservations
name: "SignerReservations"
origin: extracted
source_refs:
  - transcript:340-343
---

# SignerReservations

In-memory reservation map from OrdinalSlot to owning session id. Tracks which ordinals are live for each base agent so the allocator can pick the lowest free one and two concurrent spawns cannot both claim the same ordinal.
