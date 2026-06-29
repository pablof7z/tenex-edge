---
type: noun-entry
slug: sessionid
name: "SessionId"
origin: extracted
source_refs:
  - transcript:1930-1948
---

# SessionId

A newtype wrapping the canonical raw session id (serde-transparent). as_str() returns the raw id; its Display impl was the structural lever that routed every {session_id} format through session_codename — now flipped to render the raw id directly.
