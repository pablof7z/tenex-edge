---
type: episode-card
date: 2026-06-09
session: d8cffade-a4c3-48ab-9f29-50e8fc7e8e58
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8cffade-a4c3-48ab-9f29-50e8fc7e8e58.jsonl
salience: product
status: active
subjects:
  - nip29-group-ownership
  - daemon-session-lifecycle
  - codec-kind1
supersedes:
  - 2026-06-12-4-membership-warning-false-positive-from-stale
related_claims: []
source_lines:
  - 1-1
  - 695-696
  - 786-794
  - 877-879
  - 917-953
  - 1487-1533
  - 1700-1721
captured_at: 2026-06-12T20:07:55Z
---

# Episode: Daemon actively owns NIP-29 group per project

## Prior State

NIP-29 groups had no membership-management event handling (no 9000/9001/9007/9021/9022/39002 kinds). The daemon did not auto-create groups, auto-add agent members, or maintain a persistent ownership view. Sessions started without any group scaffolding, and agents had no guaranteed group membership when publishing presence.

## Trigger

User directive: the singleton daemon should (1) maintain an open subscription to NIP-29 groups it owns, (2) auto-create a group when starting a session in a project that lacks one, and (3) auto-add each new agent as a member so the agent finds itself already added when publishing presence. Membership events must be signed with the userNsec key.

## Decision

The daemon now actively owns one NIP-29 group per project slug. On session_start, it calls ensure_group_and_membership (create via 9007, lock via 9002 closed+public, add-member via 9000 put-user) signed by userNsec, awaited before the engine starts. A live #d-scoped subscription to 39000/39001/39002 keeps the ownership view current; handle_incoming caches kind:39002 member snapshots authoritatively. State is persisted in new owned_groups/group_members SQLite tables. Without userNsec, sessions start fail-open (group management skipped). Cache writes are gated on confirmed relay acceptance (publish_signed_checked), not best-effort, to prevent permanent phantom-ownership from transient failures.

## Consequences

- Closed+public (not private) is required: private would blind the daemon's non-member connection to its own agents' reads; closed+public enforces membership on writes while keeping reads open.
- send_event returns Ok even on relay rejection — the original best-effort cache writes would have permanently marked a nonexistent group as owned, blocking the agent forever with no self-heal. Now only confirmed-acceptance writes update the cache; failures leave it untouched so the next session_start retries.
- Once a project slug's group is owned by one operator, cross-operator peer agents that previously posted via the bare h tag get 'blocked: unknown member' — the first operator to formalize a slug becomes its gatekeeper.
- reconcile_sessions re-ensures ownership/membership for revived sessions across daemon restarts.
- f7z rate-limits group creation hard (per-IP, long window); the probe retries with backoff and skips gracefully.

## Open Tail

- Cross-operator membership negotiation (how another operator's agent joins an owned group) is not yet addressed.
- Group ownership transfer or deletion lifecycle not yet implemented.

## Evidence

- transcript lines 1-1
- transcript lines 695-696
- transcript lines 786-794
- transcript lines 877-879
- transcript lines 917-953
- transcript lines 1487-1533
- transcript lines 1700-1721

