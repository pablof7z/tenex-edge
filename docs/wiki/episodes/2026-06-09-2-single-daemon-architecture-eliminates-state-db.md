---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge-daemon
  - state-db
  - single-writer-guarantee
supersedes:
  - 2026-06-09-2-sqlite-multi-writer-corruption-is-a
related_claims: []
source_lines:
  - 707-732
  - 819-838
captured_at: 2026-06-17T23:48:53Z
---

# Episode: Single-daemon architecture eliminates state.db multi-writer corruption

## Prior State

Multiple concurrent CLI processes wrote directly to state.db, causing 16-writer corruption (proven repro). Each session was a separate process with its own database connection — no coordination, no write serialization.

## Trigger

The 16-concurrent-writer corruption repro was the load-bearing failure; the daemon design doc specified the fix (single per-machine daemon owns state.db, CLI verbs are thin UDS clients).

## Decision

Adopt single-daemon architecture: one daemon per machine owns state.db + one relay connection; every CLI verb and per-session engine are thin UDS clients. Only daemon/server.rs:119 opens state.db (single-writer proven). Versioned handshake handles binary-upgrade-under-running-daemon.

## Consequences

- 16-writer corruption structurally impossible — integrity_check ok on the repro
- All CLI verbs now go through UDS → daemon; old per-session fork is gone (__run-session removed)
- Daemon auto-spawns on first connection; stale-socket reclaim and spawn-race handling built in
- Debug-vs-release binary paths / differing TENEX_EDGE_HOME can still spawn parallel daemons (flock/socket not fully hardened) — identified as a robustness gap
- Other agents' sessions were marked dead when daemon restarted; they self-re-register on next turn

## Open Tail

- Dev-binary parallel-daemon escape hatch needs hardening (different TENEX_EDGE_HOME evades flock)

## Evidence

- transcript lines 707-732
- transcript lines 819-838

