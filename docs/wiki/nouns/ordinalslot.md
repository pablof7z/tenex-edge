---
type: noun-entry
slug: ordinalslot
name: "OrdinalSlot"
origin: extracted
source_refs:
  - transcript:327-338
---

# OrdinalSlot

A reserved ordinal slot (issue #47). At most one LIVE session per (base agent pubkey, room h, ordinal). Replaces the old binary durable-vs-transient slot: each concurrent session in a room takes the next free durable ordinal identity (smith, smith1, smith2, …), reused across rooms.
