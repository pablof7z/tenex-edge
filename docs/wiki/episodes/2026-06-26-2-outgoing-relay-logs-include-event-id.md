---
type: episode-card
date: 2026-06-26
session: b429fe81-7956-4a43-a87f-94e1799bf6e3
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b429fe81-7956-4a43-a87f-94e1799bf6e3.jsonl
salience: product
status: active
subjects:
  - relay-logging
  - event-correlation
  - observability
supersedes: []
related_claims: []
source_lines:
  - 156-190
  - 698-866
captured_at: 2026-06-26T07:57:03Z
---

# Episode: Outgoing relay logs include event ID for server correlation

## Prior State

`[→relay]` log lines showed kind, h, participants, etc., but omitted the event ID; the relay's own log includes `id=` but client-side logs could not be cross-referenced.

## Trigger

User directive: 'add to the log a matching id prefix of the event we're sending so I can correlate on the relay's log'

## Decision

Modified `log_outgoing_event` to extract and include `id=<12 hex chars>` in all outgoing relay log lines, applied consistently across all event kinds (9, 9000, 9001, 9002, 30315, etc.).

## Consequences

- Client-side and relay-server logs are now directly correlatable by event ID without external lookups
- Purely observability enhancement; no operational impact

## Open Tail

*(none)*

## Evidence

- transcript lines 156-190
- transcript lines 698-866

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-2-outgoing-relay-logs-include-event-id.json`](transcripts/2026-06-26-2-outgoing-relay-logs-include-event-id.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-2-outgoing-relay-logs-include-event-id.json`](transcripts/raw/2026-06-26-2-outgoing-relay-logs-include-event-id.json)
