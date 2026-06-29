---
type: episode-card
date: 2026-06-29
session: bd8689c8-4a5f-45b3-9dbe-758baec2a2f4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/bd8689c8-4a5f-45b3-9dbe-758baec2a2f4.jsonl
salience: architecture
status: active
subjects:
  - agent-identity
  - event-signing
  - session-keys
supersedes:
  - 2026-06-29-1-ordinal-identity-labels-flow-through-statusline
related_claims: []
source_lines:
  - 1-4
  - 488-508
  - 539-595
captured_at: 2026-06-29T10:13:55Z
---

# Episode: Each ordinal agent now signs its own events with its own key

## Prior State

Engines signed events (including identity profiles) with the durable base agent key. Ordinal sessions had a separate mechanism publishing kind:0 profiles with ordinal-derived keys, creating potential for identity overwriting when base-key publishes overlapped.

## Trigger

A bug where ordinal 1's identity profile overwrote ordinal 0's revealed the architectural flaw: the system allowed agents to share a base signing key instead of each maintaining true independence.

## Decision

Unified signing: all engines now sign events with their own key via `p.signing_keys()` (ordinal-derived key when present, base key for ordinal 0). Removed the separate ordinal kind:0 publishing block, creating one consistent signing path for all agents.

## Consequences

- Ordinal identities are now truly independent and persistent—kind:0 profiles cannot be overwritten between concurrent sessions
- Event authorship always correctly reflects the publishing agent
- System invariant: each agent signs its own identity declaration with its own key
- Eliminated redundant code paths and confusion from mixed signing models

## Open Tail

*(none)*

## Evidence

- transcript lines 1-4
- transcript lines 488-508
- transcript lines 539-595

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-each-ordinal-agent-now-signs-its.json`](transcripts/2026-06-29-1-each-ordinal-agent-now-signs-its.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-each-ordinal-agent-now-signs-its.json`](transcripts/raw/2026-06-29-1-each-ordinal-agent-now-signs-its.json)
