---
type: episode-card
date: 2026-06-17
session: e4d3c252-a2ff-40fe-b18d-a608f557322b
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e4d3c252-a2ff-40fe-b18d-a608f557322b.jsonl
salience: root-cause
status: active
subjects:
  - status-delta-dedup
  - fabric-delta-block
supersedes: []
related_claims: []
source_lines:
  - 1-293
captured_at: 2026-06-18T00:54:47Z
---

# Episode: Fabric delta block deduplicates local/peer session echo

## Prior State

`status_delta_since` in state.rs blindly unioned `session_state` (local) and `peer_session_state` (relay echoes) with no deduplication. When the daemon publishes its own session's kind:30315 status, the relay fans it back, `record_peer_status` mirrors it into `peer_session_state`, and the same session emits twice — producing the palindrome pattern visible in the user's bug report.

## Trigger

User observed duplicate session entries in the turn-start fabric block (587e5c appearing at both 27s and 18s ago), and identified the repeated-session pattern as a bug.

## Decision

Added session_id-based dedup to `status_delta_since`: local rows are collected first, their session_ids held in a HashSet, and any `peer_session_state` row whose session_id already appeared in the local set is skipped — preferring the local row, mirroring the strategy already used in `load_who_snapshot`.

## Consequences

- Local sessions no longer appear twice in delta blocks when their kind:30315 status has round-tripped through the relay.
- Self-exclusion contract is now pinned by a regression test (`status_delta_since_excludes_self_even_with_peer_echo`) proving the `exclude` param drops both the local row and the peer echo for the viewer's own session.
- Dedup is session_id-based only; `load_who_snapshot` has an additional same-host+local-pubkey guard that `status_delta_since` does not yet replicate (would require plumbing `daemon_host` into that function).
- Fix won't affect live sessions until the daemon binary is rebuilt and restarted.

## Open Tail

- Full parity with `load_who_snapshot`'s same-host+local-pubkey dedup guard requires plumbing `daemon_host` into `status_delta_since` — not yet done.
- Daemon needs restart for fix to take effect on live fabric injection.

## Evidence

- transcript lines 1-293

