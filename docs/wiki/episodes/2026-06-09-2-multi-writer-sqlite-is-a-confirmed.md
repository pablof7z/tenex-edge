---
type: episode-card
date: 2026-06-09
session: 05b89548-666c-4e24-a2f5-8a1e92f0bf04
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/05b89548-666c-4e24-a2f5-8a1e92f0bf04.jsonl
salience: root-cause
status: superseded
subjects:
  - tenex-edge-state-db
  - sqlite-multiwriter-corruption
  - per-session-persistence
supersedes: []
related_claims: []
source_lines:
  - 101-252
captured_at: 2026-06-12T19:58:43Z
---

# Episode: Multi-writer SQLite is a confirmed failure mode, not hypothetical

## Prior State

Multiple per-session __run-session engines writing one shared state.db was flagged as the biggest M1 unknown but had not yet caused a real failure.

## Trigger

state.db corrupted (SQLITE_NOTADB error 26): the main file was truncated to a single empty page while a 3.3 MB WAL from the old generation sat alongside it, producing a salt mismatch. ~16 concurrent engines were writing the same file.

## Decision

The multi-writer SQLite pattern is a confirmed, recurring failure mode — not a hypothetical risk. The architecture must change before it corrupts again. Three candidate fixes identified: (1) single-writer daemon owning state.db with IPC, (2) per-session DB files with no sharing, (3) harden shared file (busy_timeout, version pin, no truncate). Recovery completed by killing all engines and restoring state.db.bak.

## Consequences

- state.db restored from backup (integrity ok, 10 tables); corrupt originals preserved in ~/.tenex/edge/corrupt-20260609-121906/ for forensics
- Identity was never at risk — keystores live in agents/, not state.db; all lost state was ephemeral and re-populates from the relay
- .bak convention clarified: it is a previously-broken DB someone renamed per convention, NOT a deliberate clean backup — future sessions must not assume .bak = trustworthy backup
- The persistence choice now blocks the plugin bootstrap design (a single-writer daemon would mean the plugin connects to an existing daemon rather than spawning a per-session engine)

## Open Tail

- Which persistence fix to adopt (single-writer daemon vs per-session DB vs harden) remains undecided
- Plugin bootstrap design depends on the persistence choice

## Evidence

- transcript lines 101-252

