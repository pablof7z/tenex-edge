---
type: episode-card
date: 2026-06-08
session: 96aedf14-df2c-425b-b548-0fa7d1c1ba63
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/96aedf14-df2c-425b-b548-0fa7d1c1ba63.jsonl
salience: root-cause
status: active
subjects:
  - opencode-plugin
  - plugin-version-alignment
supersedes: []
related_claims: []
source_lines:
  - 1-534
captured_at: 2026-06-15T09:30:58Z
---

# Episode: opencode plugin version-mismatch caused startup hang

## Prior State

@opencode-ai/plugin was pinned at 1.2.15 in both ~/.config/opencode/ and ~/.opencode/ package.json files, while the installed opencode binary was 1.16.2 — a ~14-minor-version gap that prevented the plugin from loading, making opencode appear stuck on startup

## Trigger

user reported opencode 'just looks stuck'; investigation revealed the tenex-edge and proactive-context plugins failed to load because the plugin SDK version (1.2.15) was incompatible with the 1.16.2 runtime

## Decision

upgraded @opencode-ai/plugin to 1.16.2 in both ~/.config/opencode/package.json and ~/.opencode/package.json, and restored the wiped ~/.config/opencode/plugin/ directory with tenex-edge.ts and proactive-context.ts from their canonical sources

## Consequences

- opencode plugin package version must now stay synchronized with the opencode binary version — drifting causes silent load failure
- the plugin .ts files themselves needed no code changes; the SDK surface (info.role, info.id, info.sessionID) is compatible across the version gap
- ~/.config/opencode/plugin/ directory was found empty/wiped during re-verification, requiring manual restoration from ~/src/tenex-edge/integrations/opencode/ and ~/src/proactive-context/integrations/opencode/

## Open Tail

- no automated check ensures plugin SDK version matches the opencode binary — a future drift could reproduce the same hang
- the plugin directory wipe (cause unknown) could recur; no manifest or install script guarantees its contents

## Evidence

- transcript lines 1-534

