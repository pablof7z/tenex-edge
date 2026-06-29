---
type: noun-entry
slug: profile-domain-profile
name: "Profile (domain::Profile)"
origin: extracted
source_refs:
  - transcript:169-177
  - transcript:430-430
---

# Profile (domain::Profile)

The agent's published identity card: resolves pubkey→slug, tells a peer which machine the agent lives on, and declares the human owner(s) it belongs to (p-tagged) so a recipient can decide whether to authorize it. Encoded as kind:0 with content {"name": slug} and a ["host", host] tag.
