---
type: episode-card
date: 2026-06-09
session: 240ffb86-8827-4741-932b-29fb1824c0c7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/240ffb86-8827-4741-932b-29fb1824c0c7.jsonl
salience: product
status: active
subjects:
  - who-scope
  - nip-29-metadata
  - all-projects-flag
supersedes: []
related_claims: []
source_lines:
  - 1254-1255
  - 1452-1500
  - 1629-1633
captured_at: 2026-06-17T23:46:40Z
---

# Episode: who command defaults to current project scope with other-projects footer

## Prior State

The `who` command showed all agents across all projects regardless of where it was run, with no project grouping or summary of other projects. No NIP-29 kind 39000 (group metadata) subscription existed in the engine.

## Trigger

User requested: 'only show agents in the current project by default' with a footer listing 'x other agents in other projects: * project1 — one-liner'. The one-liner should come from NIP-29 metadata (kind 39000 `about` tag). User also requested `--project $slug` and `--all-projects` flags.

## Decision

`who` (without flags) resolves the current project from cwd and shows only agents in that project, appending a footer like '2 other agent(s) in other projects:\n  * other-project — description from NIP-29 metadata'. `--project $slug` filters to that project (with other-projects footer). `--all-projects` shows all agents flat (no footer). The engine now subscribes to kind 39000 events with `d` tag matching the project and caches `about` text in a new `project_meta` SQLite table. The live view was unified with the compact colorized renderer.

## Consequences

- New `project_meta` table in SQLite SCHEMA: `(project TEXT PRIMARY KEY, about TEXT NOT NULL, updated_at INTEGER NOT NULL)`
- New `Store` methods: `upsert_project_meta` and `get_project_meta`
- Engine (`runtime.rs`) does a one-shot fetch of kind 39000 for the current project on startup, and handles incoming kind 39000 events in `handle_incoming`
- `WhoSnapshot` now carries `current_project: String`, `all_projects: bool`, and `other_projects: Vec<OtherProjectSummary>`
- The live view (`draw_who_live`) was simplified to reuse `render_who_once` instead of a separate tabular renderer, eliminating dead code (`render_who_live`, `pad_fit`, `fit_plain`, `status_plain`)
- The `--all-projects` flag was added to the `Who` CLI subcommand

## Open Tail

- Kind 39000 subscription is project-scoped on startup; a `--all-projects` run won't show `about` descriptions for projects the engine hasn't subscribed to
- The kind 39000 handler in `handle_incoming` does inline decoding rather than going through the codec, which may need refactoring if more NIP-29 event types are added

## Evidence

- transcript lines 1254-1255
- transcript lines 1452-1500
- transcript lines 1629-1633

