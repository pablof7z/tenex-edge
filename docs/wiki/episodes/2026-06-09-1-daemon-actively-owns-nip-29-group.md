---
type: episode-card
date: 2026-06-09
session: d8cffade-a4c3-48ab-9f29-50e8fc7e8e58
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8cffade-a4c3-48ab-9f29-50e8fc7e8e58.jsonl
salience: product
status: superseded
subjects:
  - nip29-group-ownership
  - daemon-group-management
  - codec-kind1
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 785-800
  - 915-960
  - 1600-1721
captured_at: 2026-06-17T23:54:09Z
---

# Episode: Daemon actively owns NIP-29 group per project (closed+public, userNsec-signed)

## Prior State

NIP-29 groups were not actively managed by the daemon; no auto-creation, no auto-membership, no persistent group-state subscription. Groups were implicitly open and unmanaged.

## Trigger

User directive: daemon should check which NIP-29 groups we own at all times (persistent subscription), auto-create groups for new projects, auto-add agent members so agents find themselves already added when publishing presence.

## Decision

Daemon now actively owns one NIP-29 group per project. On session_start: (1) publishes 9007 create-group, (2) locks with 9002 closed+public (not private — private would blind the daemon's non-member connection to agent reads), (3) adds the agent via 9000 put-user — all awaited before the engine starts. Signed by userNsec from config. Persistent subscription to 39000/39001/39002 for owned projects added to codec filters. New owned_groups + group_members tables in state. Reconcile_sessions re-ensures ownership/membership on restart.

## Consequences

- closed+public policy enforces membership on writes while keeping reads open to non-member daemon connection
- First operator to formalize a project slug becomes its gatekeeper — cross-operator peer agents posting via bare h-tag get 'blocked: unknown member'
- Idempotent across restarts via reconcile_sessions
- Agents always find themselves already added to the group when they publish presence status
- Subscription rides the existing long-lived demux loop — no parallel subscription loop needed
- handle_incoming caches kind:39002 member snapshots authoritatively from relay-authored events

## Open Tail

- f7z rate-limits group creation hard (per-IP, long window) — many throwaway probe groups exhausted it; full f7z probe must be re-run after a creation-quiet period

## Evidence

- transcript lines 1-1
- transcript lines 785-800
- transcript lines 915-960
- transcript lines 1600-1721

