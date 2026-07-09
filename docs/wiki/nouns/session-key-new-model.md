---
type: noun-entry
slug: session-key-new-model
name: "session key (new model)"
origin: extracted
source_refs:
  - transcript:1068-1070
  - transcript:1122-1122
---

# session key (new model)

No base agent key exists; all keys are created at session start. nsec = derive(mgmt_secret, session_id), where mgmt_secret is per-machine, so the same session_id on two machines yields different keys. Every session is re-derivable and resumable from mgmt_key + session_id alone.
