---
type: noun-entry
slug: mcp-server-mosaico-mcp
name: "MCP server (mosaico mcp)"
origin: extracted
source_refs:
  - transcript:130-148
---

# MCP server (mosaico mcp)

A stateless stdio JSON-RPC loop spawned as a per-session subprocess by the harness; forwards all tool calls over a Unix domain socket to a long-running daemon. Can also serve over HTTP instead of stdio.
