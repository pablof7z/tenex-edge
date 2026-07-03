---
type: noun-entry
slug: host-neutral
name: "host-neutral"
origin: extracted
source_refs:
  - transcript:30-31
  - transcript:308-308
---

# host-neutral

Nothing inside tenex-edge knows about any host; hosts integrate from the outside via hooks and a skill. The Rust binary is the only source of truth for injected context; each host integration is a thin adapter piping JSON to tenex-edge.
