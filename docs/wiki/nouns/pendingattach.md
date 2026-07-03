---
type: noun-entry
slug: pendingattach
name: "PendingAttach"
origin: extracted
source_refs:
  - transcript:52-59
---

# PendingAttach

A struct holding a pane to attach to once the event loop yields, plus a fallback session id to resume if attaching fails because the pane is stale/gone; attaching is best-effort so a pane-not-found error never surfaces to the user.
