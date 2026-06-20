---
type: episode-card
date: 2026-06-09
session: 240ffb86-8827-4741-932b-29fb1824c0c7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/240ffb86-8827-4741-932b-29fb1824c0c7.jsonl
salience: product
status: active
subjects:
  - who-identifier-format
  - slugify-host
supersedes: []
related_claims: []
source_lines:
  - 77-80
  - 134-134
  - 1161-1170
captured_at: 2026-06-17T23:46:40Z
---

# Episode: Agent identifier in who output: slug@project → slug@hostname (slugified)

## Prior State

The `who` command displayed agents as `slug@project` (e.g. `claude@tenex-edge`), making it unclear which machine an agent was running on and conflating project scope with host identity.

## Trigger

User explicitly corrected: 'oh, that's wrong then, it should be agentSlug@hostname — hostname is provided by backend name on the .tenex/config.json file' and further noted that raw hostnames like 'pablos' laptop' would be ambiguous for agent-to-agent messaging, requiring slugification.

## Decision

Display agents as `slug@{slugify_host(host)}` where host comes from `backendName` in config (falling back to system hostname). Project is shown as a separate dimmed field. Slugification (`slugify_host`) lowercases, replaces non-alphanumeric with hyphens, collapses consecutive hyphens, and falls back to 'unknown'. The raw hostname is preserved in storage and Nostr events; slugification is display-only.

## Consequences

- Agent identifiers are now host-scoped rather than project-scoped, matching user mental model of 'which machine is this on'
- Project is still visible but as a secondary dimmed field, not in the primary identifier
- A `slugify_host` utility function was added to `util.rs` with tests
- Both the compact (`render_who_once`) and live (`draw_who_live`) renderers were updated to use the new format
- The `resolve_recipient` function still supports `slug@project` addressing for messaging, which remains project-scoped on the wire

## Open Tail

*(none)*

## Evidence

- transcript lines 77-80
- transcript lines 134-134
- transcript lines 1161-1170

