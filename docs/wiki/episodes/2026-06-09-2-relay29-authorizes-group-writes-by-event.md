---
type: episode-card
date: 2026-06-09
session: d8cffade-a4c3-48ab-9f29-50e8fc7e8e58
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8cffade-a4c3-48ab-9f29-50e8fc7e8e58.jsonl
salience: architecture
status: active
subjects:
  - relay29-authz-model
  - transport-single-connection
supersedes: []
related_claims: []
source_lines:
  - 1487-1489
  - 1544-1544
  - 1659-1675
captured_at: 2026-06-12T20:07:55Z
---

# Episode: Relay29 authorizes group writes by event author, not connection AUTH identity

## Prior State

Unknown whether relay29 authorizes NIP-29 group-management writes by the event's author key or by the connection's AUTH identity. The daemon design uses one shared relay connection authenticated as the non-member tenex-edge-daemon key, signing each event with its true author (userNsec for group management, agent key for presence). If relay29 required AUTH identity to match, the single-connection architecture would be invalid.

## Trigger

Advisor review flagged that the daemon's production topology (non-member AUTH connection signing events as the admin key) had never been tested against a real relay29 instance. nak serve doesn't enforce NIP-29, so the green integration test was a false positive for enforcement.

## Decision

Probe Q7 confirmed on a relay29 instance that an admin-signed 9000 put-user event published over a non-member-authed connection is accepted. relay29 authorizes group writes by event AUTHOR, not connection AUTH identity. The single-connection architecture is validated.

## Consequences

- The daemon's single-connection design (one relay connection, AUTHed as the daemon key, all events signed by their true author) is correct for relay29 and does not need per-key or per-member connections.
- This finding is instance-independent (relay29 is the same software everywhere), confirmed on groups.0xchat.com after f7z was rate-limited from earlier probe runs.

## Open Tail

*(none)*

## Evidence

- transcript lines 1487-1489
- transcript lines 1544-1544
- transcript lines 1659-1675

