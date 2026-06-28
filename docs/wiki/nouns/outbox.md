---
type: noun-entry
slug: outbox
name: "outbox"
origin: extracted
source_refs:
  - transcript:1409-1412
---

# outbox

durable queue of events the daemon intends to publish (status, chat, group mgmt) with retry/last-error; survives a crash between 'decide to publish' and 'relay ack'
