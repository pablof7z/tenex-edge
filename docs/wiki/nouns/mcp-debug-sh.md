---
type: noun-entry
slug: mcp-debug-sh
name: "mcp-debug.sh"
origin: extracted
source_refs:
  - transcript:524-540
---

# mcp-debug.sh

A scratchpad script that launches `tenex-edge mcp --http` fully detached from any agent-session identity (scrubbed env, reparented to launchd), fronts it with ngrok, and provides two independent traffic logs: app-level access-log JSON lines and ngrok's local inspector for full raw HTTP.
