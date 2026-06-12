---
type: episode-card
date: 2026-06-12
session: e42f09d7-5fb0-438b-a356-216870390540
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e42f09d7-5fb0-438b-a356-216870390540.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge-statusline
  - citizenship-model
  - fabric-awareness
supersedes: []
related_claims: []
source_lines:
  - 65-67
  - 200-271
  - 273-279
  - 320-329
captured_at: 2026-06-12T11:47:42Z
---

# Episode: Statusline re-anchored from generic git bar to citizenship awareness board

## Prior State

Statusline was assumed to follow standard Claude Code patterns — a generic bar showing model name, git branch, dirty marker, context burn, and cargo check results (proposals 1–10 in first round). The project's actual identity/awareness model was not reflected.

## Trigger

User correction: 'that's not anchored enough on what this project is about... read the docs'. Reading the wiki revealed the project's core thesis: agents as citizens with sovereign keypair identity, NIP-29 group membership, presence heartbeats, inbox mentions, and fleet awareness.

## Decision

The statusline is a one-line citizenship/awareness board, not a generic git bar. The agreed format is: `claude@host [session-id] ⬡N ◉N [activity] ✉ sender:message` — where ⬡N = project member count, ◉N = live session count, activity = self-reported status, ✉ = inbox envelope. Quiet when healthy; loud only on attention states (no membership, collisions, ACL pending).

## Consequences

- The statusline daemon verb must be pure-read (no state.db writes), like turn-check/peek_inbox, because Claude Code re-runs it constantly and reintroducing concurrent writers would violate the single-writer architecture doctrine
- The statusline must fail open: daemon unreachable → render just `claude@host [session-id]` rather than erroring or blocking, consistent with host adapter fail-open behavior
- Inbox segment shows pending message bright, then lingers as `✉✓` for 30s after consumption before disappearing — requires a `delivered_at` timestamp on inbox rows
- Membership warning (NOT IN GROUP) becomes a persistent statusline signal rather than relying only on injected context, which LLMs have been documented to ignore

## Open Tail

- Implementation of the `statusline` daemon RPC verb is in progress (subagent launched)
- The `delivered_at` column needed for the 30s post-consumption dimming window may not yet exist on the inbox table

## Evidence

- transcript lines 65-67
- transcript lines 200-271
- transcript lines 273-279
- transcript lines 320-329

