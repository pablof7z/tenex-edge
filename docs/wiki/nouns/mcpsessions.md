---
type: noun-entry
slug: mcpsessions
name: "McpSessions"
origin: extracted
source_refs:
  - transcript:960-966
---

# McpSessions

An in-memory map of `clientInfo.name` values self-reported at `initialize`, keyed by the session-correlation header the transport mints; unbounded — entries are never evicted, so a long-lived tunnel accumulates one row per distinct MCP client session.
