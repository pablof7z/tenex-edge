---
type: episode-card
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
salience: reversal
status: superseded
subjects:
  - channel-hierarchy
  - dotted-path-addressing
  - cli-surface
  - chat-command-removed
supersedes:
  - 2026-07-09-b70718e17221-cb4e41ba-1-full-project-vocabulary-purge-workspace-root
  - 2026-07-09-b70718e17221-cb4e41ba-2-full-project-vocabulary-purge-project-workspace
related_claims: []
source_lines:
  - 1-35
  - 139-160
  - 4142-4148
  - 4811-4818
captured_at: 2026-07-09T18:35:50Z
---

# Episode: Channel hierarchy expressed as dotted paths; `chat` command replaced by `channel read/send`

## Prior State

Channels used forward-slash hierarchy (e.g. `tenex-edge/planning`), explicitly documented as 'never dots'. The `tenex-edge chat` command existed as the primary read/send interface. Projects and channels were treated as distinct nouns — a project was a top-level NIP-29 group, a channel was the same group with a parent set.

## Trigger

User directive (lines 1-35): 'help me change how we express hierarchy to make things way easier for agents — no longer concept of projects wrt channel organization, it just happens to be nested groups.' User specified dotted-path channel addresses like `project1.epic-513.research`, replacing `tenex-edge chat` with `tenex-edge channel read`, and adding `channel join`/`channel invite` with dotted paths. User also raised that member lists must show session identity for agent-to-agent session-targeted invites to work.

## Decision

Adopted dotted-path notation for channel addresses. Removed the `chat` command entirely, rehoming its functionality under `channel read` / `channel send`. Unified the project/channel duality into one recursive node type ('channel'), with 'project' becoming a workspace-binding attribute rather than a separate concept. The `project` CLI command was removed; `list/init/edit` rehomed as `channel list --roots` / `channel init` / `channel edit <root> --about`.

## Consequences

- CLI surface changed: `chat` command gone, replaced by `channel read`/`channel send` across CLI, MCP tools, and integration tests
- Project/channel duality collapsed — one recursive `channel` node type, with workspace binding as an attribute
- GitHub issue #201 (collapse project into channel) was the tracking issue; the codebase was already ~80% unified before this session closed the gap
- MCP tool descriptions updated from 'Write a message to channel chat' to 'Send a message to a channel'
- Session-visibility-in-member-lists question raised but left as an open tail

## Open Tail

- Whether member lists now show session identity (needed for agent-to-agent session-targeted invites) — raised at line 21 but not clearly resolved in visible transcript
- MCP lacks a `channel_add` tool despite the shipped verb (tracked as #320)

## Evidence

- transcript lines 1-35
- transcript lines 139-160
- transcript lines 4142-4148
- transcript lines 4811-4818

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-1-channel-hierarchy-expressed-as-dotted-paths.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-1-channel-hierarchy-expressed-as-dotted-paths.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-channel-hierarchy-expressed-as-dotted-paths.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-1-channel-hierarchy-expressed-as-dotted-paths.json)
