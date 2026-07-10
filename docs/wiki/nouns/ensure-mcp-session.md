---
type: noun-entry
slug: ensure-mcp-session
name: "ensure_mcp_session"
origin: extracted
source_refs:
  - transcript:2235-2235
  - transcript:2907-2907
  - transcript:2977-2978
  - transcript:2845-2854
---

# ensure_mcp_session

The one-time registration RPC that mints a real keypair for an MCP HTTP caller via the same derive_session_keys_v2 scheme every hosted session uses, then writes an ordinary session row tagged harness="mcp" — mirroring rpc_session_start's resolve-or-mint → mint_session_identity → write-row sequencing but without spawning an OS process. Gated behind --oauth so an unauthenticated endpoint can never mint identities.
