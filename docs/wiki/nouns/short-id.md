---
type: noun-entry
slug: short-id
name: "short_id"
origin: extracted
source_refs:
  - transcript:40-46
---

# short_id

A short prefix of a message/event id (its first 6 hex chars) — cheap to include inline in agent-facing context. The daemon resolves any unambiguous prefix back to the full event, so it stays round-trippable without spending tokens on a full 64-char id.
