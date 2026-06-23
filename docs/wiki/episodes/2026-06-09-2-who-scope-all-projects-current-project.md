---
type: episode-card
date: 2026-06-09
session: 240ffb86-8827-4741-932b-29fb1824c0c7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/240ffb86-8827-4741-932b-29fb1824c0c7.jsonl
salience: product
status: superseded
subjects:
  - who-command
  - project-scoping
  - nip-29-metadata
supersedes: []
related_claims: []
source_lines:
  - 1161-1280
  - 1629-1633
  - 1452-1498
captured_at: 2026-06-12T19:59:53Z
---

# Episode: who scope: all-projects → current-project default with other-projects footer

## Prior State

`who` showed all agents across all projects regardless of which directory it was run from, with no project grouping or filtering. No NIP-29 metadata integration existed.

## Trigger

User's original question about project visibility, then explicit request: "only show agents in the current project by default" with footer format specification and "for the one-liner we'll use nip29 metadata event for that channel -- if no metadata exists then leave the one-liner description empty"

## Decision

`who` defaults to current project (resolved from cwd via `project::resolve`). Main list shows only that project's agents. Footer lists other projects with agent counts and NIP-29 kind 39000 `about` descriptions. `--all-projects` flag added to show all agents flat (no footer). Engine subscribes to kind 39000 events and caches `about` text in new `project_meta` SQLite table.

## Consequences

- `WhoSnapshot` gained `other_projects: Vec<OtherProjectSummary>` and `project: String`
- `load_who_snapshot` signature changed to `current_project: Option<&str>` (None = all projects)
- New `project_meta` table in SQLite: `(project TEXT PRIMARY KEY, about TEXT NOT NULL, updated_at INTEGER NOT NULL)`
- Engine adds kind 39000 subscription on startup with one-shot fetch + live handler
- `--all-projects` CLI flag added
- Live view unified: both once and live renderers now use same `render_who_once` colorized output
- Tabular plain-text renderer (`render_who_live`) removed

## Open Tail

*(none)*

## Evidence

- transcript lines 1161-1280
- transcript lines 1629-1633
- transcript lines 1452-1498

