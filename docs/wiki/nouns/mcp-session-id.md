---
type: noun-entry
slug: mcp-session-id
name: "Mcp-Session-Id"
origin: extracted
source_refs:
  - transcript:452-453
---

# Mcp-Session-Id

An HTTP-transport session-tracking id minted at MCP `initialize` time (`mcp-<nanos>-<counter>`, matching the existing mint_session_id convention), used to key clientInfo.name in an in-memory map so later requests from the same connection can recover it; issued and returned via the `Mcp-Session-Id` response header.
