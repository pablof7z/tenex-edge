---
type: episode-card
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
salience: reversal
status: superseded
subjects:
  - project-wording-purge
  - workspace-concept
  - root-channel
  - vocabulary-purge
  - hook-render
supersedes:
  - 2026-07-09-b70718e17221-cb4e41ba-1-project-concept-fully-purged-replaced-by
  - 2026-07-09-b70718e17221-cb4e41ba-1-project-cli-command-removed-functions-rehomed
related_claims: []
source_lines:
  - 4160-4189
  - 4274-4323
  - 4573-4577
  - 4678-4693
captured_at: 2026-07-09T17:55:38Z
---

# Episode: Full project-wording purge: project → workspace / root channel across entire codebase

## Prior State

After removing the `project` CLI command (#305), the 'project' vocabulary remained pervasive: the `<project name="tenex-edge">` hook wrapper injected into agent context every turn, statusline 'Project:' line, `who` output ('project blocks', 'project tabs'), internal module `crate::project`, `project_roots` table, `project_add`/`project_list`/`project_edit` RPC methods, `--project`/`--all-projects` CLI flags. ~1596 occurrences across 191 files. Issue #201 had deliberately kept 'project' in human-facing rendering as 'project root'.

## Trigger

User correction at line 4160: 'didn't we say we were abandoning projects wording? I still see many things with project.' User then chose full purge scope including internals (line 4187: 'Everything incl. internals').

## Decision

Full vocabulary purge splitting 'project' into two concepts: **workspace** (machine+path binding, the former 'project root') and **root channel** (parent-empty top-of-tree channel). 184 files changed: `<project name>` → `<workspace name>` in hook render, `crate::project` → `crate::workspace`, `project_roots` table → `workspace_roots` (with row-preserving migration), RPC renames (`project_add`→`channel_add_member`, `project_members`→`channel_members`, `project_remove`→`channel_remove_member`, `project_list`→`root_channels`, `project_edit` deleted as orphaned), `--project`/`--all-projects` → `--root`/`--all-roots`, statusline/who now say 'Root:' / 'other root channels'. NIP-29 wire tags untouched.

## Consequences

- Human-facing agent context now emits `<workspace name="X">` with zero 'project' in any rendered surface (grep-verified on merged master)
- Database migration: `project_roots` table rows copied to `workspace_roots` and old table dropped; `~/.tenex-edge/projects.json` → `workspaces.json` with legacy read-fallback
- RPC wire surface changed (method names renamed) — accepted wire skew, all callers + integration tests updated
- Reconciled with concurrent PR #324 (zephyr's agentSlug work) — git auto-merged all but one test assertion, hand-merged to keep both identity line and workspace rename
- 891 unit + 46 e2e tests green on merged master e94a6f3f
- Legacy 'project' wording intentionally retained only in: migration code (backward-compat table/file names), negative-guard tests (asserting removed command stays removed), and the English verb 'projects' (data projection, not the noun)

## Open Tail

- Nothing deployed to live fleet — running daemon still emits `<project>` until explicit rollout (runs workspace_roots migration + identity migration on live state.db, restarts sessions)
- #320: MCP server has no `channel_add` tool despite the shipped verb (net-new surface gap)

## Evidence

- transcript lines 4160-4189
- transcript lines 4274-4323
- transcript lines 4573-4577
- transcript lines 4678-4693

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-3-full-project-wording-purge-project-workspace.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-3-full-project-wording-purge-project-workspace.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-3-full-project-wording-purge-project-workspace.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-3-full-project-wording-purge-project-workspace.json)
