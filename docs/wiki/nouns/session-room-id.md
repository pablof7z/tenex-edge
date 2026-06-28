---
type: noun-entry
slug: session-room-id
name: "session_room_id"
origin: extracted
source_refs:
  - transcript:885-892
---

# session_room_id

A deterministic ID for a per-session room: 'session-' followed by six base36 alphanumeric chars derived from a stable hash of the session's anchor (resume token/harness id/pid); the 'session-' prefix is the explicit, canonical marker
