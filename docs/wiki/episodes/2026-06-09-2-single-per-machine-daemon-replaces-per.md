---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge
  - daemon-architecture
  - state-db-ownership
supersedes:
  - 2026-06-09-2-multi-writer-sqlite-is-a-confirmed
related_claims: []
source_lines:
  - 709-710
  - 819-838
  - 857-862
captured_at: 2026-06-12T20:02:14Z
---

# Episode: Single per-machine daemon replaces per-session state.db writers

## Prior State

Each CLI invocation and per-session engine opened state.db independently, causing multi-writer corruption (reproducible with 16 concurrent writers). Per-session fork processes each owned their own relay connection.

## Trigger

16-concurrent-writer corruption repro proven; architectural necessity for the channel wake path (which needs a persistent process to receive events).

## Decision

Single per-machine daemon owns state.db + one relay connection. Every CLI verb and per-session engine is now a thin UDS client. Only daemon/server.rs opens state.db (single-writer guarantee proven).

## Consequences

- Multi-writer state.db corruption is structurally impossible — only daemon/server.rs:119 opens it.
- Versioned handshake handles binary-upgrade-under-running-daemon without data loss.
- Spawn-race, stale-socket reclaim, and version-skew (daemon-exit + client-respawn) all tested.
- Dev workflow hazard: debug-vs-release binaries or differing TENEX_EDGE_HOME can evade the flock/socket lock and spawn parallel daemons — same multi-instance failure class the daemon was built to kill.
- The old per-session binary is replaced; currently running sessions need restart to re-exec against the daemon.

## Open Tail

- Parallel-daemon hardening (flock/socket lock bypass via different binaries/homes) is a known gap.

## Evidence

- transcript lines 709-710
- transcript lines 819-838
- transcript lines 857-862

