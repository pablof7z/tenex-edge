---
type: noun-entry
slug: session-as-identity-handle
name: "session (as identity handle)"
origin: extracted
source_refs:
  - transcript:547-547
  - transcript:1068-1068
  - transcript:1122-1122
---

# session (as identity handle)

A single run (SESSION) is only a correlation handle (the raw session_id); it is never a separate display name and is never accepted as a chat target. Under the new model, the session_id is the sole input to key derivation — nsec = derive(mgmt_secret, session_id).
