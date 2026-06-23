# tenex-edge — consolidated architecture plan

Source material: `architecture-plan-opus.md` (Opus Architect) and
`architecture-plan-codex.md` (codex exec), produced independently from the same
brief and converging on the same diagnosis. This doc is the merged, decision-locked
plan. Where they differed, the chosen option is marked **[LOCKED]** or **[REC]**.

## Diagnosis (both agreed)

Every recurring bug is one bug: **there is no `Session` aggregate with a single
write authority, and the daemon borrows session identity from the harness instead
of minting it.** One logical fact — *session S is titled T, doing A, busy B, alive
L* — physically lives in runtime-local `cur_title`/`cur_activity`
(`runtime.rs:149`), `session_status`, legacy `agent_status`, `turn_state`, and the
kind:30315 tag, written from ~7 scattered sites with no single transaction. And
`session_id` is simultaneously the sqlite PK, the relay `d`-tag, the routing
target, and the harness resume token — origin unstable (opencode mints a new one
every start). Combined with kind:30315 never expiring, identity rotation × permanent
events = unbounded competing title events.

## Locked decisions

- **[LOCKED] Liveness = NIP-40 expiration on the status heartbeat.** kind:30315
  carries `["expiration", now + status_ttl]`, re-armed every heartbeat. Session
  stops → beats stop → event expires → reads as dead. TTL = 90s (`status_ttl`),
  heartbeat = 30s (3× re-arm margin, no flicker); 90s already equals the `who`
  peer-freshness window so local and peer liveness use one number. Reverses commit
  `5e7a34d1`. Accepted consequence: a finished session's title leaves the relay
  ~90s after its last beat. No tombstones / no `lifecycle: ended/superseded` events.
- **[LOCKED] Scope = full session aggregate** (through legacy-table resolution).
- **[REC] Runtime = stateless driver.** `run_session_in_daemon` loses
  `cur_title`/`cur_activity` and becomes a pure-ish `on_tick(now, &store) ->
  Vec<Effect>` (Opus shape — maximally table-testable), reading the persisted row
  and emitting effects (publish, schedule-distill). It never holds the only copy of
  state and never builds `DomainEvent::Status` directly.
- **[REC] Versioned transitions + status outbox.** Every status-changing transition
  bumps a `state_version` and enqueues a `status_outbox(session_id, state_version)`
  row; a daemon drainer publishes kind:30315 (with expiration), records the native
  event id, retries. Distill results apply only if `(turn_id, base_version)` still
  match → stale distills and duplicate runtimes structurally cannot flip the title.

## Target architecture

**Identity.** Daemon mints a stable canonical `session_id`. Harness ids, resume
ids, generated `te-*` ids, and tmux pane ids become rows in `session_aliases`.
`rpc_session_start` delegates the id decision to a registry:
`register_or_reassert_session(observation) -> SessionSnapshot` — alias hit →
existing id; same `(harness, agent, project, host, pane|resume_id|watch_pid)` live
→ reattach; new logical session on the same pane/pid → supersede old in one txn.
Only the hook boundary (`cli/hooks.rs`) speaks native ids; it reports *normalized
observations*, not identity policy.

**State = one row.** `session_state` (canonical `session_id` PK): title,
title_source, activity, phase/busy, turn_id, turn_started_at, last_distill_at,
last_seen, resume metadata, state_version. All mutation through transition methods
on `Store` (`start_turn`, `seed_title_if_empty`, `apply_distill_result`,
`heartbeat`, `end_turn`, `end_session`, `supersede_session`), each one SQLite txn
that also enqueues the outbox when public status changed. `session_status`,
`agent_status`, `turn_state` stop being independently writable.

**Derivation.** One `derive_status(state, now) -> DerivedStatus` shared by the
publisher, both `who.rs` branches, `rpc_statusline`, and the turn delta. Kills the
local-vs-peer busy-flag fork.

**Local vs peer split.** Local → `session_state` (keyed by canonical id). Peer →
`peer_session_state` (keyed by pubkey/project/native), materialized from inbound
kind:30315. The materializer physically cannot write local state, so the `is_self`
guard in `fabric/mod.rs` stops being load-bearing.

**Ownership rules (explicit, enforced).** Local session state: SQLite
authoritative, relay is a projection. Peer status: relay authoritative input,
SQLite is the read model. NIP-29 membership: relay authoritative, SQLite a cache.
Messages/threads: canonical sqlite authoritative for UI/delivery. Compatibility
tables are projections written by one canonical path only — never independently.

## Migration (incremental, testable; no rewrite)

- **Phase 0 — Freeze invariants as failing-first tests.** Extend the FREEZE suite +
  `tests/daemon_integration/*`: one `d` per logical session across id rotation;
  title stable across turn_end + daemon restart; concurrent session_start → one
  runtime + one status address; re-fired opencode (same resume_id/pane) → same id;
  stale distill ignored; `who` busy flag identical local vs peer; expired status not
  counted live.
- **Phase 1 — Canonical session tables + read facade.** Add `session_state`,
  `session_aliases`, `status_outbox`; backfill from existing tables on open; add
  read methods. Keep old writes. Prove the facade represents today's state.
- **Phase 2 — Identity into the registry.** `rpc_session_start` →
  `register_or_reassert_session`; feed the canonical id into `spawn_session`'s
  atomic reserve; write alias rows. Closes the identity-rotation bug class.
- **Phase 3 — Transitions own title/activity/busy + versioned distill.** Move status
  writes out of `runtime.rs` into `Store` transitions (may dual-write old tables
  during migration); runtime calls transitions and receives snapshots.
- **Phase 4 — Status outbox + provider status API + expiration.** Add
  `Kind1Nip29Provider::set_status` and the outbox drainer; emit the NIP-40
  expiration tag here; runtime stops calling `provider.publish(Status)` directly.
- **Phase 5 — Switch local readers to canonical.** `who`, `build_status_delta`,
  `rpc_statusline`, `build_backfill` onto `derive_status` + the facade.
- **Phase 6 — Finish message/inbox canonical migration.** Move
  `route_mention_into` out of `runtime.rs`; materializer writes canonical first,
  derives legacy inbox from it.
- **Phase 7 — Convert legacy tables to projections or delete.** `agent_status`,
  `session_status`, `project_meta`, `group_members`, `inbox`/`chat_inbox` →
  views/projections or removed; each table's retained/deleted status made
  unambiguous in `state.rs`.
- **Phase 8 — Simplify.** Fix `DaemonState.last_status` keyed by `(pubkey,project)`
  (wrong for multi-session → key by session id); drop dead `EngineParams` fields;
  tail derivation reads the canonical projection.

## Keep (both agreed)

Single-writer daemon; "no await while holding the Store lock"; pure `domain.rs`;
the kind:30315 self-contained per-session *shape*; provider/codec seam; NIP-29
relay-authoritative membership; canonical origin tables + idempotent
materialization; publish-once-route-locally in `rpc_send_message`; the atomic
`spawn_session` reservation (kept as a runtime-supervisor guard below the registry);
the hook adapter registry (fail-open, but reporting observations not identity).

## Success criteria

One sql row answers "current title/activity/busy/lifecycle"; the relay event is a
projection of it, never a competing source; re-firing a hook for the same logical
session returns the same id; a new session on the same pane supersedes the old in
one txn; all readers go through canonical methods; legacy tables are gone or
compatibility-only; a duplicate runtime or stale distill cannot flip the title;
dead sessions disappear from the relay within `status_ttl` via expiration.
