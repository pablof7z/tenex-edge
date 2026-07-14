---
type: noun-entry
slug: publish-de
name: "publish_de"
origin: extracted
source_refs:
  - transcript:490-504
---

# publish_de

Historical runtime closure that published `DomainEvent`s with the wrong signing
keys. Current runtime publishing signs with the key selected for the session's
authoritative pubkey and carried through `EngineParams`.
