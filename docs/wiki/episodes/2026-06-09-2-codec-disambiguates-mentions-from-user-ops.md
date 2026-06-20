---
type: episode-card
date: 2026-06-09
session: 98f9939c-f42b-43dd-baba-d9a176d4b2d7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/98f9939c-f42b-43dd-baba-d9a176d4b2d7.jsonl
salience: product
status: active
subjects:
  - kind1-codec
  - mention-activity-disambiguation
  - feedback-loop-prevention
supersedes: []
related_claims: []
source_lines:
  - 2289-2306
captured_at: 2026-06-17T23:52:07Z
---

# Episode: Codec disambiguates Mentions from user OPs by requiring agent tag

## Prior State

Any kind:1 Nostr event with a `p` tag was decoded as a Mention (routed to the tagged agent's inbox), regardless of whether the event was authored by an agent or a human

## Trigger

Adding the user prompt hook meant user-originated kind:1 events would carry a `p` tag (targeting the agent) but no `agent` tag — they would be decoded as Mentions and routed back to the agent, creating a feedback loop

## Decision

The codec's kind:1 decode now requires the `agent` tag to be present for Mention classification. Events with a `p` tag but no `agent` tag decode as Activity instead, preventing user-originated prompts from appearing as agent-directed mentions.

## Consequences

- User OPs (kind:1, p-tagged, no agent tag) are now decoded as Activity, not Mention — no feedback loop
- Agent-to-agent Mentions (which always carry both `agent` and `p` tags) are unaffected
- This is now the canonical domain rule for disambiguating kind:1 events in the codec

## Open Tail

*(none)*

## Evidence

- transcript lines 2289-2306

