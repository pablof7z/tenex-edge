---
type: episode-card
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
salience: reversal
status: superseded
subjects:
  - project-wording-purge
  - workspace-binding
  - root-channel
  - rpc-renames
  - table-migration
  - hook-wrapper
supersedes:
  - 2026-07-09-b70718e17221-cb4e41ba-1-channel-hierarchy-expressed-as-dotted-paths
related_claims: []
source_lines:
  - 4160-4189
  - 4255-4272
  - 4274-4324
  - 4670-4693
captured_at: 2026-07-09T18:35:50Z
---

# Episode: Full 'project' vocabulary purge — project → workspace / root channel across all surfaces

## Prior State

Issue #201 deliberately retained 'project' in human-facing rendering (as 'project root') and in all internal names (crate::project, project_roots table, project_add RPC, <project name="X"> hook wrapper). The earlier #305 PR removed only the `project` *command* but left the 'project' *vocabulary* almost entirely intact — including the <project name="tenex-edge"> wrapper injected into agent context every turn.

## Trigger

User correction (line 4160): 'didn't we say we were abandoning projects wording? I still see many things with project.' The assistant acknowledged under-delivering: when it removed the project command (#305), it left the project vocabulary almost entirely intact. User then chose 'Everything incl. internals' for purge scope (line 4187).

## Decision

Full purge of 'project' vocabulary across both human-facing and internal surfaces. Two replacement concepts: **workspace** (machine+path binding) and **root channel** (parent-empty top-of-tree channel). 184 files changed: `crate::project` → `crate::workspace`; `project_roots` table → `workspace_roots` (with row-preserving migration); RPC methods renamed (`project_add`→`channel_add_member`, `project_members`→`channel_members`, `project_list`→`root_channels`, `project_edit` deleted as orphaned); `<project name="X">` hook wrapper → `<workspace name="X">`; CLI flags `--project`/`--all-projects` → `--root`/`--all-roots`; statusline/who output changed to `Root:` / 'other root channels'. NIP-29 wire tags left untouched (no 'project' on the wire).

## Consequences

- Rendered agent context has zero 'project' wording — verified by grep on merged master e94a6f3f
- 891 unit + 46 e2e tests green after purge; on-disk `projects.json` → `workspaces.json` with legacy read-fallback for migration
- Reconciled cleanly with concurrent PR #324 (agent-slug rendering) — git auto-merged all but one test assertion, resolved by hand
- RPC wire surface changed: old method names are gone, callers updated across CLI and integration tests
- Table migration (`workspace_roots_migration.rs`) copies legacy `project_roots` rows into `workspace_roots` and drops old table; fresh/already-migrated DBs no-op
- Reply-envelope design-doc contradiction filed as #322: docs still say 'sibling sessions share a pubkey' which per-session keys inverted — left as code-verified follow-up rather than guessing
- Daemon not yet deployed to live fleet — running daemon still emits old `<project>` wrapper until explicit rollout

## Open Tail

- #320: MCP has no `channel_add` tool despite the shipped verb
- #322: daemon-design §8a/§8b reply-envelope rationale still says sibling sessions share a pubkey — needs code-verified doc rewrite
- Live fleet deployment of the new daemon (runs identity + workspace_roots migrations on live state.db) — left as user's explicit call

## Evidence

- transcript lines 4160-4189
- transcript lines 4255-4272
- transcript lines 4274-4324
- transcript lines 4670-4693

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-2-full-project-vocabulary-purge-project-workspace.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-2-full-project-vocabulary-purge-project-workspace.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-2-full-project-vocabulary-purge-project-workspace.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-2-full-project-vocabulary-purge-project-workspace.json)
