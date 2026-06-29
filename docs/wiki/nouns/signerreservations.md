---
type: noun-entry
slug: signerreservations
name: "SignerReservations"
origin: extracted
source_refs:
  - transcript:340-343
---

# SignerReservations

In-memory reservation map from OrdinalSlot to owning session id. Tracks which ordinals are live in each room so the allocator can pick the lowest free one and two concurrent spawns can't both claim the same ordinal.
