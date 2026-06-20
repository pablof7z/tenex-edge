---
type: episode-card
date: 2026-06-08
session: 96aedf14-df2c-425b-b548-0fa7d1c1ba63
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/96aedf14-df2c-425b-b548-0fa7d1c1ba63.jsonl
salience: root-cause
status: active
subjects:
  - opencode-plugin-version-coupling
supersedes: []
related_claims: []
source_lines:
  - 1-165
  - 165-238
  - 387-414
  - 525-534
  - 556-605
captured_at: 2026-06-17T23:40:57Z
---

# Episode: opencode plugin version must match binary version

## Prior State

The @opencode-ai/plugin dependency was pinned at v1.2.15 across both plugin install locations (~/.config/opencode/ and ~/.opencode/) while the opencode binary had been upgraded to v1.16.2 — the version mismatch caused the plugin to silently fail to load, making opencode appear stuck on startup.

## Trigger

User reported opencode looking 'stuck' when run; investigation revealed opencode v1.16.2 vs plugin package v1.2.15 — a ~14-minor-version gap. The new SDK adds `effect` as a dependency, indicating significant API surface changes. The plugin directory (~/.config/opencode/plugin/) was also found wiped during re-configuration.

## Decision

Upgrade @opencode-ai/plugin to 1.16.2 in both install locations and restore the plugin .ts files (tenex-edge.ts, proactive-context.ts) to ~/.config/opencode/plugin/. The plugin source code itself required no changes — the types used (info.role, info.id, info.sessionID) are stable across versions.

## Consequences

- opencode and @opencode-ai/plugin are version-coupled: any future opencode upgrade must include a matching plugin package bump across all install directories
- There are two separate install locations for the plugin package (~/.config/opencode/ and ~/.opencode/) that must be kept in sync
- The canonical plugin sources live outside the opencode config tree (in ~/src/tenex-edge/integrations/opencode/ and ~/src/proactive-context/integrations/opencode/) and must be manually restored if the plugin directory is cleared

## Open Tail

- No automated version-sync mechanism exists — future opencode upgrades risk recurring version mismatch if plugin deps aren't co-updated
- No guardrail prevents the plugin directory from being wiped again

## Evidence

- transcript lines 1-165
- transcript lines 165-238
- transcript lines 387-414
- transcript lines 525-534
- transcript lines 556-605

