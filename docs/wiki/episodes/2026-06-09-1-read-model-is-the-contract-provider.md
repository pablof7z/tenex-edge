---
type: episode-card
date: 2026-06-09
session: d208c058-7b2b-4ff8-bb82-d63623d51097
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d208c058-7b2b-4ff8-bb82-d63623d51097.jsonl
salience: architecture
status: superseded
subjects:
  - fabric-architecture
  - read-model
  - cqrs
  - provider-seam
supersedes: []
related_claims: []
source_lines:
  - 675-714
captured_at: 2026-06-17T23:57:01Z
---

# Episode: Read model is the contract; provider is write-side materializer

## Prior State

Draft fabric-architecture had reads flowing through the provider; the materializer was framed as re-owning decode/subscribe capabilities; the store was described as greenfield

## Trigger

User corrected that reads must never go through the provider — the provider is purely a write-side materializer and how data is hydrated is invisible to readers

## Decision

Reframed the architecture: reads query the unified store directly (no provider in the call path); the provider/materializer composes codec + delivery rather than re-owning them; the store extends the existing state.db rather than replacing it

## Consequences

- ACL gate is consulted twice over the same store rows — once as write-side admission predicate, once as read-side query — never on the wire
- Materializer owns only admit + derive + upsert; decode and subscribe are not re-owned
- threads moved from open questions to a resolved store noun the materializer derives
- Single-writer daemon identified as the direct fix for multi-writer corruption

## Open Tail

- Thread keying across fabrics (root id vs. synthesized hash vs. subject)
- Write-reflection timing (optimistic vs. echo)

## Evidence

- transcript lines 675-714

