---
type: noun-entry
slug: mcp-session-anchor
name: "mcp_session anchor"
origin: extracted
source_refs:
  - transcript:2905-2912
---

# mcp_session anchor

A new CallerAnchor kind, resolved by the existing lookup function exactly like pty_session or harness_session — read-only, never creates on its own. After ensure_mcp_session registers the caller, every tool call forwards the same hint so all handlers resolve through that one shared path.
