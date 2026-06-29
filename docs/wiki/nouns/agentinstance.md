---
type: noun-entry
slug: agentinstance
name: "AgentInstance"
origin: extracted
source_refs:
  - transcript:921-928
---

# AgentInstance

The single authoritative identity value for a session, carrying base_slug, base_pubkey, ordinal, and pubkey, with methods display_slug(), agent_ref(), signing_keys(&base_keys). The single place base-vs-ordinal policy lives; created at session birth and threaded through EngineParams, replacing the distributed identity state across session rows, identity rows, and in-memory signer maps.
