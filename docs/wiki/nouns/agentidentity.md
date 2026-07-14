---
type: noun-entry
slug: agentidentity
name: "AgentIdentity"
origin: extracted
source_refs:
  - transcript:102-105
---

# AgentIdentity

The resolved agent configuration used when selecting a session signer. Ordinary
sessions allocate pubkey-owned reconstruction material. Agents configured with
`perSessionKey:false` instead use the configured key across sequential runs.
See [Mosaico Agent Identity](../guides/mosaico-agent-identity.md).
