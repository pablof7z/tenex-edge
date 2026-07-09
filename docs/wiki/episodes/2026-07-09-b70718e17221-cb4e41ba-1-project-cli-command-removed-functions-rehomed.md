---
type: episode-card
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
salience: reversal
status: superseded
subjects:
  - project-command-removal
  - channel-cli-surface
  - project-channel-unification
supersedes: []
related_claims: []
source_lines:
  - 141-159
  - 3897-3936
  - 3967-3972
captured_at: 2026-07-09T17:55:38Z
---

# Episode: Project CLI command removed, functions rehomed under channel

## Prior State

Projects and channels were distinct nouns — a project was a top-level NIP-29 group owning a workspace, a channel was the same group with a parent. The CLI had a separate `project` command surface (`project list`, `project init`, `project edit`). Issue #201 tracked collapsing the project concept entirely, and the codebase was already ~80% collapsed (one resolver, identity = (parent, name)), but the duality still leaked into daemon/CLI/docs.

## Trigger

User's initial directive to simplify hierarchy for agents and remove the 'projects' concept wrt channel organization (lines 1-5), plus issue #201 ('Collapse project into channel: one recursive node, workspace binding as an attribute').

## Decision

Removed the `project` CLI command entirely. `project list` → `channel list --roots` (new `--roots` flag), `project init` → `channel init`, `project edit --description` → `channel edit <root> --about`. Deleted `Cmd::Project` variant, `ProjectAction` enum, and `project_admin.rs` file. Kept underlying RPCs (`project_add`, `project_list`, etc.) intact per instructions, though `rpc_project_edit` became orphaned from CLI.

## Consequences

- `channel list --roots` and `channel list` (subtree) are now mutually exclusive flag contexts — capability preserved via flag, not separate command
- `channel edit` on a root channel now requires an agent session (Strict anchor resolution), whereas old `project edit` could run from a bare project dir by slug — potential regression for human-from-cwd root editing
- `rpc_project_edit` is orphaned from the CLI (only caller was `project edit`); flagged for later cleanup
- `crate::project` slug-resolution module kept intact (used by launch/who workspace resolver)
- Shipped as PR #321, merged to master (878 tests green)

## Open Tail

- Orphaned `rpc_project_edit` RPC still in daemon dispatch but unreachable from CLI
- Whether human-from-cwd root editing needs restoring (channel edit now requires agent session)

## Evidence

- transcript lines 141-159
- transcript lines 3897-3936
- transcript lines 3967-3972

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-1-project-cli-command-removed-functions-rehomed.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-1-project-cli-command-removed-functions-rehomed.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-project-cli-command-removed-functions-rehomed.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-project-cli-command-removed-functions-rehomed.json)
