---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: architecture
status: superseded
subjects:
  - session-identity
  - state-ownership
  - session-key
supersedes: []
related_claims: []
source_lines:
  - 1037-1088
  - 1091-1165
captured_at: 2026-06-16T11:19:04Z
---

# Episode: Architectural rethink: daemon-owned session identity and single state source

## Prior State

Session state has no single owner: title/activity/busy is scattered across 4 stores (task-local cur_title, session_status.text, agent_status, kind:30315 tag) written from ~7 call sites with no apply(transition) chokepoint — CQRS inverted (volatile task frame is write model, sqlite is lossy mirror). Session identity is borrowed from the harness (claude/codex adopt their own id, opencode mints a new one every start), and combined with kind:30315's never-expire design, identity rotation × permanent events = unbounded competing title events by construction.

## Trigger

User declared 'everything feels incredibly buggy and stitched together — state is all over the place' after three related title/status bugs, directing an architectural research effort via Opus and Codex agents.

## Decision

Adopt two structural invariants: (1) daemon-minted stable session_key as the only identity — promote the existing stale-sibling predicate (agent+project+host+watch_pid) from cleanup heuristic to THE identity function; harness native ids become aliases in a session_aliases table; (2) single session_state row (session_key PK) as source of truth for title/activity/phase/turn timing, with all mutations flowing through one commit_session_state(store, key, StateTransition) function. Six-phase incremental migration: freeze tests → add session_key+aliases → move d-tag to session_key → introduce session_state → extract derive_status() → split peer_state.

## Consequences

- Title-drift, orphaned-event, and status-flip bug classes become structurally unrepresentable rather than patched
- SessionDriver becomes a stateless on_tick(now, store) → Vec<Effect> — table-testable transition over the persisted row
- derive_status() shared by publisher, who (both branches), and turn-context delta — deletes local-vs-peer fork
- Atomic spawn_session reservation becomes belt-and-suspenders rather than the only guard against flip-flop
- Codex research agent's independent plan still pending — synthesis deferred until both land

## Open Tail

- Codex exec architectural plan not yet written — cross-validation pending
- User has not yet reviewed or approved the Opus plan
- Three tactical fixes (atomic spawn, prompt-seed, distill timing) are in the working tree but uncommitted — may be superseded or subsumed by the rethink phases

## Evidence

- transcript lines 1037-1088
- transcript lines 1091-1165

