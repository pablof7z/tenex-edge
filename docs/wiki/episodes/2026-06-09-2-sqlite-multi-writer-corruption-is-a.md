---
type: episode-card
date: 2026-06-09
session: 05b89548-666c-4e24-a2f5-8a1e92f0bf04
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/05b89548-666c-4e24-a2f5-8a1e92f0bf04.jsonl
salience: root-cause
status: superseded
subjects:
  - tenex-edge-state-db
  - sqlite-multiwriter
  - persistence-architecture
supersedes: []
related_claims: []
source_lines:
  - 101-252
captured_at: 2026-06-17T23:45:24Z
---

# Episode: SQLite multi-writer corruption is a confirmed failure mode

## Prior State

Per-session __run-session processes all write one shared state.db SQLite file. This multi-writer risk was flagged as the biggest M1 unknown but had not yet manifested.

## Trigger

state.db was found truncated to a single empty page (schema 0) with a 3.3 MB orphaned WAL — producing SQLITE_NOTADB (error 26). ~16 concurrent engines were writing the same file. Most likely one engine truncated/recreated the base file mid-flight.

## Decision

Incident recovered by killing all engines and restoring state.db.bak (which was a previously-broken DB someone renamed, not a deliberate backup — but verified integrity-ok with 10 tables). Three architectural fixes identified but not yet chosen: (1) single-writer daemon owning state.db, sessions talk IPC; (2) per-session DB files, no sharing; (3) harden shared file (enforce binary version, busy_timeout, no truncate/VACUUM).

## Consequences

- Multi-writer SQLite corruption is a confirmed, recurring failure mode — restoring the backup buys time but it will happen again without an architectural fix.
- Identity was never at risk — keystores live in ~/.tenex/edge/agents/, not in state.db.
- Everything in state.db is ephemeral/reconstructible from the live relay (presence, agent_status, seen_mentions, turn_state, pending agents).
- The .bak file convention is a broken DB rename, not a deliberate backup — future sessions should not assume '.bak = backup'.
- Corrupt originals preserved in ~/.tenex/edge/corrupt-20260609-121906/ for forensics.

## Open Tail

- Architectural fix not yet chosen — single-writer daemon vs per-session DB vs hardening.
- This decision intersects with the CC plugin question: a single-writer daemon changes what the plugin's bootstrap hook starts/connects-to.

## Evidence

- transcript lines 101-252

