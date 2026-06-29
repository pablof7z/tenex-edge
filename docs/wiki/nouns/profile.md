---
type: noun-entry
slug: profile
name: "Profile"
origin: extracted
source_refs:
  - transcript:169-182
---

# Profile

The agent's published identity card. Resolves pubkey to slug, tells a peer which machine the agent lives on, and declares the human owner(s) it belongs to (p-tagged), so a recipient can decide whether to authorize it. Encoded as kind:0 with content {"name": slug}.
