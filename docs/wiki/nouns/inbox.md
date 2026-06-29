---
type: noun-entry
slug: inbox
name: "inbox"
origin: extracted
source_refs:
  - transcript:635-640
---

# inbox

The inbound routing ledger AND the idempotency record. One row per (inbound event, target local session). An event is "handled" because a row exists; there is no separate processed-orchestration table. A row starts `pending` and becomes `delivered` once injected into a live tmux pane.
