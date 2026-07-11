---
type: noun-entry
slug: sessionidentity
name: "SessionIdentity"
origin: extracted
source_refs:
  - transcript:2546-2548
---

# SessionIdentity

A lean read-side struct containing the pubkey, agent slug, session id, and
friendly code. `display_slug()` returns `agent-session-code`, and `agent_ref()`
returns the same public handle with its pubkey. Used for routing, rendering, and
member display.
