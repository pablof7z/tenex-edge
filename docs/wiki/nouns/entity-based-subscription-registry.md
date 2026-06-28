---
type: noun-entry
slug: entity-based-subscription-registry
name: "entity-based subscription registry"
origin: extracted
source_refs:
  - transcript:446-447
  - transcript:1392-1410
---

# entity-based subscription registry

subscription model that plans the daemon's relay subscriptions around entities (channels via #h, ordinal pubkeys via #p, groups via #d) with narrow add-REQs for new entities, replacing the previous per-(project×kind) model; introduces real CLOSE/unsubscribe to prevent subscription leaks and drops kind:0 profiles to fetch-on-demand
