---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: architecture
status: active
subjects:
  - session-identity
  - session-state
  - session-aggregate
  - daemon-architecture
supersedes:
  - 2026-06-16-2-architectural-rethink-daemon-owned-session-identity
related_claims: []
source_lines:
  - 1037-1090
  - 1092-1251
captured_at: 2026-06-16T11:28:20Z
---

# Episode: Session aggregate architecture: daemon-minted identity and single source of truth

## Prior State

Session state was scattered across 4+ stores (runtime-local cur_title/cur_activity, session_status table, legacy agent_status table, kind:30315 tag) written from ~7 scattered sites with no single commit/transition chokepoint. session_id was borrowed from the harness (unstable: opencode mints a new one every start), serving simultaneously as sqlite PK, relay d-tag, routing target, and harness resume token. Combined with kind:30315 never expiring, identity rotation × permanent events produced unbounded competing title events by construction.

## Trigger

User directed: 'everything feels incredibly buggy and stitched together — take a step back and rethink the architecture.' Two independent research agents (Opus Architect, Codex) both converged on the same diagnosis.

## Decision

Adopt full session aggregate: daemon-minted stable session_key as the only identity (harness ids become aliases in a session_aliases table); one session_state row as single source of truth (title, title_source, activity, phase, turn_started_at, etc.) mutated only through explicit transition methods; runtime task becomes a stateless SessionDriver (on_tick → Vec<Effect>); one deterministic derive_status() projection shared by who, statusline, and the publisher; separate local vs peer state so the materializer cannot write local state.

## Consequences

- Makes title-drift, orphaned-events, and status-flip structurally unrepresentable rather than patched
- 6-phase incremental migration starting with FREEZE invariant tests
- Tactical fixes from this session (atomic spawn, prompt-seed, distill timing) are consistent and become belt-and-suspenders under the new model
- Dual-write to legacy tables (agent_status, session_status) eventually deleted; canonical tables (threads, messages) kept only where they have real read paths

## Open Tail

- Phase sequencing and PR boundaries not yet specified
- Whether to adopt Codex's status_outbox for publish retry/idempotency or keep inline publishing

## Evidence

- transcript lines 1037-1090
- transcript lines 1092-1251

