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
  - channel-hierarchy
supersedes:
  - 2026-07-09-b70718e17221-cb4e41ba-3-full-project-wording-purge-project-workspace
related_claims: []
source_lines:
  - 4160-4162
  - 4187-4187
  - 4274-4323
  - 4574-4576
  - 4678-4693
captured_at: 2026-07-09T18:00:57Z
---

# Episode: Full "project" vocabulary purge → workspace + root channel

## Prior State

Issue #201 deliberately kept the word "project" in human-facing rendering (as "project root") and in all internal names (crate::project, project_roots table, project_add RPC, --project CLI flag). The project command had been removed (#305) but the vocabulary persisted everywhere — the <project name="tenex-edge"> hook wrapper injected into agent context every turn, statusline "Project:" line, who output, RPC method names, table names, module names.

## Trigger

User pointed out: "didn't we say we were abandoning projects wording? I still see many things with 'project'" (line 4160). The assistant confirmed 1596 occurrences across 191 files and acknowledged it under-delivered by only removing the command, not the vocabulary. User then chose full scope: "Everything incl. internals" (line 4187).

## Decision

Complete purge of "project" vocabulary from both human-facing surfaces AND internal identifiers. Two replacement concepts: "workspace" for the machine+path binding attribute, and "root channel" for the parent-empty top-of-tree channel. Specifically: <project name> → <workspace name> in hook output; crate::project → crate::workspace; project_roots table → workspace_roots (with row-preserving migration); project_add → channel_add_member, project_members → channel_members, project_list → root_channels, project_edit deleted; --project/--all-projects → --root/--all-roots; projects.json → workspaces.json with legacy read-fallback. NIP-29 wire tags untouched (no "project" on the wire).

## Consequences

- 184 files changed across the codebase; 891 unit + 46 e2e tests green on merged master
- Row-preserving SQLite migration (project_roots → workspace_roots) required for live state.db; on-disk map projects.json → workspaces.json with one-time legacy fallback
- RPC wire surface changed (project_add → channel_add_member etc.) — wire skew accepted; all dispatch, CLI callers, and integration tests updated
- Reconciled with concurrent PR #324 (agent-slug rendering) — git auto-merged all but one test assertion, resolved by hand; both changes coexist on merged master
- Must rebase other agents' work (e.g. @flint-range-108's fabric fixes in profiles.rs/messaging/chat_read_tail) onto renamed symbols when landing
- Not deployed to live fleet — daemon still emits <project> until explicit rollout; migration + restart required

## Open Tail

- #322 — daemon-design §8a/§8b reply-envelope rationale still claims shared pubkey (contradicts per-session keys, see separate arc)
- #320 — MCP server has no channel_add tool despite the shipped verb
- Live-fleet deployment pending user's explicit trigger (runs workspace_roots migration on live state.db)
- @flint-range-108's uncommitted fabric fixes need rebasing onto the renamed codebase

## Evidence

- transcript lines 4160-4162
- transcript lines 4187-4187
- transcript lines 4274-4323
- transcript lines 4574-4576
- transcript lines 4678-4693

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-1-full-project-vocabulary-purge-workspace-root.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-1-full-project-vocabulary-purge-workspace-root.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-full-project-vocabulary-purge-workspace-root.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-full-project-vocabulary-purge-workspace-root.json)
