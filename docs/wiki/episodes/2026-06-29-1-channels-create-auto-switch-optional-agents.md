---
type: episode-card
date: 2026-06-29
session: b20ef4ab-0b54-4770-a549-4ed195c0035e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b20ef4ab-0b54-4770-a549-4ed195c0035e.jsonl
salience: product
status: active
subjects:
  - channels-create
  - auto-switch
  - agent-optional
  - parent-resolution
  - dedup-to-error
supersedes: []
related_claims: []
source_lines:
  - 1-3
  - 253-260
  - 590-617
  - 618-653
captured_at: 2026-06-29T10:51:54Z
---

# Episode: channels create: auto-switch, optional agents, current-channel parent, hard-error on duplicate

## Prior State

`channels create` required a `--project` (cwd-resolved) parent and at least one `--agent` target. It returned the existing channel id silently when the name already existed (dedup path with `deduped: true`). There was no auto-switch into the newly created channel — the creating agent's session stayed on its prior channel.

## Trigger

User directive at lines 1-3: 'when an agent creates a channel it should auto switch to it' and '--agent should not be required'. Follow-up directive at line 618: 'it should error out if that channel already exists so the agent knows about it'.

## Decision

Four behavior changes to `channels create`: (1) After provisioning the new room, the daemon re-homes the creating agent's session into it via a shared `rehome_session_to_channel` helper (extracted from `channels switch`). (2) `--agent` is now optional; with zero targets, the channel is created/joined but no kind:9 add-agents orchestration event is published. (3) Parent resolution changed: `--parent-channel <ref>` (project-relative) takes precedence, then the creator's current channel is the default (replacing cwd-resolved `--project`), with an explicit literal `parent` kept for the launch picker and tests. (4) The dedup path (duplicate channel name) is now a hard error instead of a silent return — the agent gets an actionable error message so it knows the channel already exists.

## Consequences

- Creator resolution uses the strict (no project-fallback) path so child-of-current and auto-switch only fire for genuine agent sessions.
- Auto-switch on create is unconditional for agent sessions (no occupancy/membership guards needed since creator is sole member).
- The dedup-to-error change means a repeat `channels create --name X` now surfaces a failure to the agent rather than silently succeeding — agents must handle this error or check existence first.
- `rehome_session_to_channel` shared helper prevents drift between create and switch code paths.
- Pre-existing 3-test failures in channels suite are unrelated (mid-refactor of `session_start.rs` by another workstream).
- New integration test `channels_create_no_agents_nests_under_current_and_auto_switches` validates all three initial behaviors; existing `channels_create_auto_creates_missing_parent_project` still passes.

## Open Tail

- Duplicate-error test was being added at end of session — not yet confirmed passing.
- Wiki channel-creation guide docs may need updating to reflect new parent-resolution and dedup-error semantics.

## Evidence

- transcript lines 1-3
- transcript lines 253-260
- transcript lines 590-617
- transcript lines 618-653

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-channels-create-auto-switch-optional-agents.json`](transcripts/2026-06-29-1-channels-create-auto-switch-optional-agents.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-channels-create-auto-switch-optional-agents.json`](transcripts/raw/2026-06-29-1-channels-create-auto-switch-optional-agents.json)
