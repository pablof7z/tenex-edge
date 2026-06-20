---
title: tenex-edge Session Label
slug: tenex-edge-session-label
topic: tenex-edge
summary: The session label is split into a persistent title (`Status.text`) and a separate `active`/`idle` boolean, rather than using a single status field that is wiped
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-17
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:f0f28929-320e-4608-96bd-6f8ff7e0d602
  - session:84aaaa96-2082-42b1-b9af-83f9c8f90f67
  - session:215d979a-a054-4e2b-b349-851e0d874d6d
  - session:622711fa-5176-4580-b311-d66446c2924b
  - session:rollout-2026-06-14T13-19-49-019ec5a5-1119-76f0-a7e3-36bc985a31bd
  - session:rollout-2026-06-16T12-56-00-019ecfdb-f9ce-72b1-b465-398423cae745
  - session:rollout-2026-06-16T17-43-45-019ed0e3-68e5-7091-899d-6a4e0fcb5716
  - session:rollout-2026-06-17T11-06-26-019ed49e-0783-7c50-a1db-0850a653f66c
  - session:ses_13aa6b531ffefKa6oNJXbEUoHk
---

# tenex-edge Session Label

## Session Label Architecture

The session label is split into a persistent title (`Status.text`) and a separate `active`/`idle` boolean, rather than using a single status field that is wiped to empty on turn-end.

A single LLM call produces both the stable session title and the live activity line using a combined prompt that outputs two labeled lines: TITLE and NOW. The separate live-activity distiller (`summarize_via_rig` / `RIG_SYSTEM_PROMPT`) and its wiring are removed.

Session titles must describe the session's overall objective rather than the current step, preferring the user's stated request or goal (e.g. 'Fix GitHub issue 1', not 'Finding the issue repository'). The TITLE field uses nudge-to-keep logic, repeating the current title verbatim unless the objective substantively changed, while the NOW field is always regenerated each turn.

When a new turn is detected (rising edge) and the session has no title yet, the engine immediately seeds the title from the user's message (titleized and truncated) before any LLM distillation call.

The label parser is tolerant: case-insensitive, accepting ACTIVITY/DOING as synonyms for NOW, stripping trailing punctuation, and treating a bare line as the title.

The `who` command displays the activity line alongside the title when the session is busy (e.g. 'Fix GitHub issue 1 — reading the issue tracker · busy') and collapses to just the title and idle flag when idle (e.g. 'Fix GitHub issue 1 · idle').

The title persists across idle turns while the session is alive and is cleared only on session exit. The live activity is cleared on idle and at the start of each new turn, making it ephemeral and not recovered across daemon restarts.

Engine triggers re-distillation on the rising edge of a new turn (new user message), keeps the title on the falling edge (idle), rather than using periodic 30s/5min timers.

When no model is configured, `distill_title` falls back to the existing title rather than leaving it empty.

`turn_repeat` is kept as an optional in-turn re-distill safety valve defaulting to off, since each new user message already re-distills.

The in-memory title is repopulated from the store on engine startup so a daemon restart still feeds the existing title back to the distiller.

A single `derive_status()` projection is shared by `who`, the status line, and delta calculations.

A `status_outbox` plus a provider `set_status` mechanism exists, and the runtime is a stateless driver that never builds `DomainEvent::Status`.

Newly appeared sessions in the "changes since your last turn" hook render their actual title and status label, not just "joined".

Identical session reasserts refresh liveness without bumping `updated_at`, preventing them from appearing as changed in turn deltas. Repeated idle/end observations refresh liveness without re-emitting the session as a change.

Status change deltas include the session ID in the format `↻ {slug}@{proj} [session {sid}] — {text}`, with `[session {sid}]` omitted when the session ID is None for legacy `agent_status` rows. `list_status_changes_since` returns `Vec<(String, String, String, Option<String>)>` — (slug, project, text, session_id) — with session_id being NULL for legacy `agent_status` rows and the actual session_id for `session_status` rows. Both `state.rs` and `state/inbox.rs` implementations of `list_status_changes_since` are updated to include the session_id column, and both `cli/who.rs` and `cli.rs` delta formatting are updated to include the session ID.

The legacy `agent_status` and `session_status` tables are deleted with no backward compatibility. Backward-compatibility handling is removed; decode reads the active tag directly with no legacy text-inference fallback or serde(default).

The `active` boolean is carried over the wire as an `["active","0"|"1"]` tag.

The session title is carried in a `['title', '...']` tag on the NIP-29 kind:30315 status event, not in the event's `content` field. The `content` field represents live activity and is intentionally empty when a session is idle; a consumer inspecting only the `content` field will see sessions as titleless even though the title is present in the `title` tag. The activity field is published as an optional `['activity', '...']` tag on the same event, omitted when empty (e.g. on idle), maintaining backward compatibility with older peers.

Stale sessions from older slug configurations (e.g. 'claude') can be republished with empty titles if their daemon is still heartbeating them as alive after the title logic changed.

`is_idle()` is determined by `!active` rather than empty text.

The "[no tmux]" label is not shown in the TUI because all local sessions are resumable.

Projects with no live sessions and no activity in the past 12 hours are hidden by default.

Session title and activity labels are persisted and rendered live, with idle rows retaining titles and active rows showing 'title — activity' format.

<!-- citations: [^62271-3] [^62271-4] [^rollo-51] [^f0f28-1] [^f0f28-2] [^f0f28-3] [^f0f28-4] [^f0f28-5] [^f0f28-6] [^f0f28-7] [^f0f28-8] [^f0f28-9] [^f0f28-10] [^f0f28-11] [^84aaa-1] [^215d9-5] [^rollo-60] [^rollo-87] [^rollo-112] [^ses_1-3] -->
