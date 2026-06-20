---
type: episode-card
date: 2026-06-09
session: d8cffade-a4c3-48ab-9f29-50e8fc7e8e58
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8cffade-a4c3-48ab-9f29-50e8fc7e8e58.jsonl
salience: root-cause
status: active
subjects:
  - relay29-authz-model
  - nip29-topology
  - daemon-connection-auth
supersedes: []
related_claims: []
source_lines:
  - 1489-1491
  - 1544-1578
  - 1656-1677
captured_at: 2026-06-17T23:54:09Z
---

# Episode: Relay29 authorizes group writes by event author, not connection AUTH identity

## Prior State

Uncertain whether relay29 authorizes group-management writes by the event's author key or the connection's AUTH identity. The daemon uses one relay connection authed as the non-member tenex-edge-daemon key but signs each event with the operator's userNsec. nak serve (local test relay) doesn't enforce NIP-29, so integration tests are false-green for enforcement topology.

## Trigger

Advisor review identified this as an untested topology (Blocker 1): the production authz model was unverified, and a wrong assumption would invalidate the entire single-connection design.

## Decision

Empirically verified via Q7 probe on relay29 (groups.0xchat.com) that an admin-signed 9000 put-user over a non-member-authed connection is accepted. relay29 authorizes by event author, not connection AUTH identity. The daemon's single non-member connection with per-event author signing is sound.

## Consequences

- Single connection authed as tenex-edge-daemon is architecturally valid — no need for per-key connections or member-AUTHed connections
- Design validated for the full management path (create, lock, put-user)
- Probe Q7 added to nip29_probe.rs as a permanent topology regression check

## Open Tail

*(none)*

## Evidence

- transcript lines 1489-1491
- transcript lines 1544-1578
- transcript lines 1656-1677

