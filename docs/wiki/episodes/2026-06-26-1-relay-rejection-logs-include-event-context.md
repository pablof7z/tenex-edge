---
type: episode-card
date: 2026-06-26
session: b429fe81-7956-4a43-a87f-94e1799bf6e3
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b429fe81-7956-4a43-a87f-94e1799bf6e3.jsonl
salience: product
status: active
subjects:
  - relay-logging
  - rejection-context
supersedes: []
related_claims: []
source_lines:
  - 1-154
captured_at: 2026-06-26T07:57:03Z
---

# Episode: Relay rejection logs include event context

## Prior State

Rejection log lines like `[relay✗] rejected: no relay returned OK (timeout)` contained only the reason string, with no information about which event was rejected, making correlation to user actions impossible.

## Trigger

User complaint: 'I need to know what was rejected or what timedout -- I have no idea what event is not getting accepted from that'

## Decision

Modified `log_relay_rejection` signature to accept `Option<&Event>`, and updated the three call sites in `transport.rs` that have access to the signed event to pass it through. Rejection lines now emit `kind:N  id=<12 hex>  h=<group>` when the event is available.

## Consequences

- `publish_builder_checked` (no event in scope) continues emitting reason-only rejections, creating a two-tier logging format
- `publish_signed_checked` and `publish_event_checked` now emit self-contained rejection lines with full event context

## Open Tail

- publish_builder_checked callers cannot correlate rejections without additional context

## Evidence

- transcript lines 1-154

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-1-relay-rejection-logs-include-event-context.json`](transcripts/2026-06-26-1-relay-rejection-logs-include-event-context.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-1-relay-rejection-logs-include-event-context.json`](transcripts/raw/2026-06-26-1-relay-rejection-logs-include-event-context.json)
