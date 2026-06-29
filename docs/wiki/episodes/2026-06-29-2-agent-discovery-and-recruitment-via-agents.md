---
type: episode-card
date: 2026-06-29
session: 661ebf6b-e01b-4ff6-b9c7-5042b900c788
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/661ebf6b-e01b-4ff6-b9c7-5042b900c788.jsonl
salience: product
status: active
subjects:
  - tenex-edge-agents
  - tenex-edge-invite
  - agent-recruitment
supersedes: []
related_claims: []
source_lines:
  - 2936-3280
captured_at: 2026-06-29T10:05:11Z
---

# Episode: Agent discovery and recruitment via `agents` roster and `invite` command

## Prior State

No dedicated command to list invitable agents. Agent recruitment implicit via @-mention (no auto-spawn) or `tenex-edge launch` (requires explicit channel). No awareness of remote-backend agents.

## Trigger

User's proposal included 'Agents: List of agents you can invite' as first-class output, with explicit mention of `invite` as 'the explicit alternative to @-mentioning, which never auto-spawns'.

## Decision

Built two commands: (1) `tenex-edge agents` — lists invitable roster with bylines, reads keystore directly, no daemon required. (2) `tenex-edge invite <slug[@backend]>` — spawns fresh session pinned to inviter's current channel (explicit alternative to implicit @-mention). Extracted `invite_slug` parsing to testable helper supporting both bare slug and `slug@backend` remote specs.

## Consequences

- New commands establish standard discovery→recruitment flow (query roster → invite explicitly)
- `invite` RPC handler reuses `spawn_agent` logic but forces channel to inviter's current context
- Backend-aware agent identifiers now threaded through stack (enables invoking agents on remote backends via `@` syntax)
- Agents command verified against real keystore (bylines from kind:30315 events)
- Exit code and output format allow scripting around agent availability

## Open Tail

- No fuzzy picker for agent disambiguation — always exact slug match
- No per-backend filtering of roster

## Evidence

- transcript lines 2936-3280

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-2-agent-discovery-and-recruitment-via-agents.json`](transcripts/2026-06-29-2-agent-discovery-and-recruitment-via-agents.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-2-agent-discovery-and-recruitment-via-agents.json`](transcripts/raw/2026-06-29-2-agent-discovery-and-recruitment-via-agents.json)
