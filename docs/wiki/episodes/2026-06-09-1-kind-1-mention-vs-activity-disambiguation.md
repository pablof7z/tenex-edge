---
type: episode-card
date: 2026-06-09
session: 98f9939c-f42b-43dd-baba-d9a176d4b2d7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/98f9939c-f42b-43dd-baba-d9a176d4b2d7.jsonl
salience: product
status: active
subjects:
  - codec-kind1
  - mention-routing
  - user-op
supersedes: []
related_claims: []
source_lines:
  - 2249-2306
captured_at: 2026-06-12T20:06:37Z
---

# Episode: Kind:1 Mention vs Activity disambiguation by agent tag

## Prior State

Any kind:1 event carrying a `p` tag was decoded as a Mention, routing it to the p-tagged agent's inbox — regardless of author identity or whether an `agent` tag was present.

## Trigger

Adding a user OP hook that publishes kind:1 events with a `p` tag (tagging the processing agent) but no `agent` tag. Under the old codec logic, these user-originated messages would be decoded as Mentions and looped back into the agent's inbox.

## Decision

The decode path now requires both a `p` tag AND an `agent` tag to classify a kind:1 event as a Mention. Events with a `p` tag but no `agent` tag fall through to Activity classification. This means user OPs (no `agent` tag) are decoded as Activity rather than Mention, preventing the feedback loop.

## Consequences

- User-originated kind:1 OPs are no longer routed to agent inboxes
- Agent-originated kind:1 events with both `p` and `agent` tags remain Mentions
- Any future kind:1 event type that includes a `p` tag but no `agent` tag will default to Activity semantics
- The `agent` tag becomes the definitive marker distinguishing agent-to-agent Mentions from other kind:1 events

## Open Tail

- If a third kind:1 subtype needs different routing (e.g. user-to-user messages), the `agent` tag heuristic may need further refinement

## Evidence

- transcript lines 2249-2306

