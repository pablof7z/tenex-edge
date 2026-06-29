---
type: episode-card
date: 2026-06-29
session: 661ebf6b-e01b-4ff6-b9c7-5042b900c788
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/661ebf6b-e01b-4ff6-b9c7-5042b900c788.jsonl
salience: product
status: active
subjects:
  - roster-delta
  - hook-context
  - awareness-injection
supersedes: []
related_claims: []
source_lines:
  - 3286-3370
captured_at: 2026-06-29T10:05:11Z
---

# Episode: Roster-change delta — surface new agents automatically in turn context

## Prior State

When new agents added to keystore, no mechanism to surface availability in turn context. Agents had to manually query `tenex-edge agents` or discover via @-mention.

## Trigger

User's proposal included automatic roster presentation in agent-context output, implying new agents should be captured and surfaced to the running session.

## Decision

New agents surface in hook context (the fabric-context injection) only when created since session's last turn. Reuses existing cursor: `created_at > session.turn_started_at`. No new schema required.

## Consequences

- Roster emergence is deterministic and idempotent (bounded by session turn boundaries)
- Pure formatter `new_agent_block` is injectable and unit-tested; daemon fs-read confined to `turns.rs` RPC layer
- Prevents roster flood on first turn (only new-since-last-turn agents appear)
- Reuses existing `created_at` metadata from identity module — no new tracking

## Open Tail

- Does not surface removed agents (only additions)
- Does not track per-agent 'first-seen' — uses session turn boundary (may group multiple agents created together)

## Evidence

- transcript lines 3286-3370

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-3-roster-change-delta-surface-new-agents.json`](transcripts/2026-06-29-3-roster-change-delta-surface-new-agents.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-3-roster-change-delta-surface-new-agents.json`](transcripts/raw/2026-06-29-3-roster-change-delta-surface-new-agents.json)
