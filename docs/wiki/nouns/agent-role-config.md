---
type: noun-entry
slug: agent-role-config
name: "agent role config"
origin: extracted
source_refs:
  - transcript:656-656
  - transcript:808-815
---

# agent role config

Launchable roles come from native Codex, Claude Code, and OpenCode agent
definitions plus optional Mosaico overrides under
`<mosaico_home>/agents/<slug>.json`. Native definitions are monitored globally
and per bound workspace, so they need no duplicate JSON. An explicit JSON binds
an ambiguous role to a harness bundle or requests a config profile or durable
`perSessionKey:false` identity. Ordinary session keys are derived from the
machine's management key and are not stored in agent JSON.
