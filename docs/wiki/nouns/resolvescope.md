---
type: noun-entry
slug: resolvescope
name: "ResolveScope"
origin: extracted
source_refs:
  - transcript:2067-2076
---

# ResolveScope

An enum controlling how far session resolution may reach past exact anchors. `Strict` = exact anchors only (PTY session, harness id, explicit override), fails loud rather than binding a sibling — used for per-session mutations. `Channel` = exact anchors then cwd+agent scan (latest-alive in channel) — used for reads and host-facing commands.
