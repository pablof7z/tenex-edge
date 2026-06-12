---
type: episode-card
date: 2026-06-09
session: 98f9939c-f42b-43dd-baba-d9a176d4b2d7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/98f9939c-f42b-43dd-baba-d9a176d4b2d7.jsonl
salience: root-cause
status: active
subjects:
  - nip29-groups
  - daemon-relay
  - group-creation
supersedes: []
related_claims: []
source_lines:
  - 2496-2898
captured_at: 2026-06-12T20:06:37Z
---

# Episode: NIP-29 groups were absent because installed daemon used old relay default

## Prior State

The wiki and code state that `ensure_group_and_membership` auto-creates NIP-29 groups on `rpc_session_start` / `reconcile_sessions`. Despite 66 sessions for `tenex-edge`, both `owned_groups` and `group_members` SQLite tables were empty. The user expected the `tenex-edge` group to already exist on the relay.

## Trigger

Running `tenex-edge project list` returned only probe test groups, not `tenex-edge`. Investigation showed the running daemon binary (installed at `~/.local/bin/tenex-edge`) was the pre-update build, connecting to the old default relay `wss://relay.tenex.chat` instead of `wss://nip29.f7z.io`. The daemon log showed `userNsec unset; skipping NIP-29 group management` from older runs.

## Decision

After `just install` (rebuilding and reinstalling the binary) and restarting the daemon, `reconcile_sessions` ran on startup and successfully created both `tenex-edge` and `tenex-off` groups on `nip29.f7z.io`. The group auto-creation code works correctly; the issue was purely a stale binary.

## Consequences

- Protocol version was bumped from 1 to 2 during the session, so future daemon/client version skew will trigger automatic re-exec
- Any deployment must ensure `just install` is run before expecting new RPCs or relay changes to take effect
- The two-relay split (`relay.tenex.chat` for auth-gated subscriptions vs `nip29.f7z.io` for NIP-29 group management) is confirmed in practice

## Open Tail

- Presence events (kind:30315) now flow to `nip29.f7z.io`, which enforces group membership — auto-creation must succeed before presence can land

## Evidence

- transcript lines 2496-2898

