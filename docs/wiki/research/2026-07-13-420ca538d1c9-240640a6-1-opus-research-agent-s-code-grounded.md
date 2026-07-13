---
type: research-record
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
source_lines: 665-741
agent_attribution: Opus research agent (a2dcd80ecdb85982b)
has_preregistered_criteria: false
has_method: true
has_structured_report: true
characterization: "Opus research agent's code-grounded investigation of non-PTY session delivery consequences — 8 consequences (C1–C8) each verdicted CONFIRMED, with feasibility analysis for three planned fixes"
captured_at: 2026-07-13T07:30:52Z
---

Opus research agent's code-grounded investigation of non-PTY session delivery consequences — 8 consequences (C1–C8) each verdicted CONFIRMED, with feasibility analysis for three planned fixes

---

User: [Agent task result: Agent "Research non-PTY session consequences" finished]
I now have complete evidence. Here is my report.

---

# Consequences of a session NOT being daemon-PTY-wrapped

## 1. Mechanism summary

**What "PTY-wrapped" means here.** A session is "PTY-wrapped" iff the daemon spawned its harness inside its own portable-pty supervisor. The supervisor sets `TENEX_EDGE_PTY_SESSION` in the child's env (`src/pty/supervisor.rs:51`). The session-start hook reads that env var (`src/cli/hooks/observation.rs:33`) and reports it to the daemon, which records a `pty_session` **alias** row for the session. Endpoint identity lives ONLY in the `aliases` table under kind `"pty_session"` — the `sessions` table has `child_pid` but **no pane/pty column** (`src/state/schema/ddl.rs:141`, cols listed `src/state/sessions.rs:10-12`). A human-launched `codex --yolo resume` in iTerm never has `TENEX_EDGE_PTY_SESSION` set, so `pty_session` is `None` (`observation.rs:33`) and **no `pty_session` alias is ever written**. The hook still registers the session (its `child_pid` is found by walking the process tree, `observation.rs:88-103` / `cli/hooks.rs:298-302`), so the row exists and is `alive=1`.

**Two delivery paths:**

- **(A) Active-turn self-drain (PTY-independent).** Every time the harness fires a hook (UserPromptSubmit → `turn_start`, Stop/PostToolUse → `turn_check`), the daemon assembles turn context and calls `take_inbox` → `claim_pending_for_session`, which atomically flips all `pending` rows to `delivered` and returns them for inline injection into that turn (`src/turn_context/reads.rs:32-38`, called at `src/turn_context/start.rs:151` and `src/turn_context/check.rs:43`). This works for ANY session, PTY-wrapped or not, **as long as the agent keeps taking turns** — because the harness itself drives the hook.

- **(B) Idle doorbell injection (PTY-only).** When the agent is idle (no hooks firing), the only way to deliver is `ring_doorbells` → the daemon types the rendered mention into the PTY and hits Enter (`src/session_host/delivery/prompt.rs:80` + `:128`), which both delivers the text and wakes the agent to self-drain. This path resolves the endpoint from the `pty_session` alias (`src/session_host/delivery.rs:163-167`) and requires a **live** PTY.

The observed split ("delivered while turning, black-holed once idle") is exactly path A working and path B being unavailable.

## 2. Enumerated consequences of NOT being PTY-wrapped

**C1 — Idle mentions silently black-hole (CONFIRMED, root cause).** With no `pty_session` alias, `ring_doorbells` resolves `pty_id = None` (`delivery.rs:163-167`). The delivery reconciler's `decide()` hits the `None` branch and returns `DeferNoEndpoint` (`src/reconcile/delivery/model.rs:184-192`). `translate()` maps `DeferNoEndpoint` to **zero effects** (`src/reconcile/delivery/mod.rs:238` — literally `=> {}`). Result: pending rows stay `pending`, `delivered_at=0`, forever. No inject, **no retry scheduled, no failure emitted, nothing logged as a problem.** This is the silent black-hole.

**C2 — Delivery depends entirely on the agent voluntarily taking a turn (CONFIRMED).** The only working path is A (`reads.rs:35`). Nothing external can trigger a turn on a non-PTY session, so if the human doesn't type into the iTerm tab, queued mentions are never surfaced.

**C3 — The offline-mention recovery that WOULD fix this is suppressed by the alive flag (CONFIRMED).** `offline_mention::dispatch` is called unconditionally on every first-sight p-tagged chat to a local pubkey (`src/daemon/server/demux.rs:170`). But `handle()` early-returns as soon as `list_alive_sessions()` shows any alive session for that (pubkey, channel) (`src/daemon/server/demux/offline_mention.rs:42-59`). Because the iTerm session is `alive=1`, the handler bails **before** it would `resume_agent` into a daemon-owned PTY (`offline_mention.rs:190` / `:221`). So the one mechanism that could re-home the agent into an injectable endpoint is short-circuited precisely because the un-injectable session looks reachable. The 236 "dispatching offline-agent-mention handler" log lines (`offline_mention.rs:19-23`) are this dispatch entry firing once per mention event and then no-oping.

**C4 — Presence / `who` shows the agent online while it is undeliverable (CONFIRMED).** Liveness is pure PID existence: `pid_alive` = `kill(pid, None).is_ok()` (`src/daemon/server/engine_lifecycle.rs:263-265`). The heartbeat keeps `last_seen` fresh (`touch_session`, `src/state/sessions.rs:234-240`; status re-arm `src/state/status.rs:29-39`). The reaper marks dead only on `stale` (last_seen too old) **or** `process_dead` (child_pid gone) (`src/daemon/server/membership_cleanup.rs:82`). The iTerm codex pid is alive and heartbeating, so **neither condition ever fires** — the zombie-but-online row persists indefinitely and `who`/presence lie.

**C5 — Doorbell edge-trigger is missed with no fallback (CONFIRMED).** `ring_doorbells` is edge-triggered — fired from `demux.rs:191` when `outcome.wake_mentions`, and re-armed only via `RetryAfter`/`DeferDebounced` self-reschedule (`delivery.rs:220-223,232-240`). Since a non-PTY session yields `DeferNoEndpoint` (no effect, no `RetryAfter`), a missed edge is never retried. There is no level-triggered sweep that re-attempts stuck `pending` rows for endpoint-less sessions.

**C6 — First-turn chat replay may also be skipped (CONFIRMED, secondary).** `replay_chat = active && (channel_already_subscribed || pty_session.is_some())` (`src/reconcile/session_start/mod.rs:290`). For a fresh non-PTY session whose channel isn't already subscribed, chat replay is gated off at start — a second way inbound context can be missed at registration time.

**C7 — Class-B idle nudge never delivered (CONFIRMED, minor).** The "are you done?" nudge requires `pty_session` env and a live PTY (`src/cli/hooks/class_b.rs:4-20`), so non-PTY ephemeral sessions are never nudged to clean themselves up — reinforcing C4's zombie longevity.

**C8 — The undeliverable state is never surfaced to any human (CONFIRMED).** `emit_delivery_failure` emits a `TailEvent::delivery_failure` (`src/daemon/server.rs:125-139`) only on actual errors inside the doorbell loop. The `DeferNoEndpoint` path emits none, so no tail line, no `who` marker, no log warning ever tells an operator that messages are piling up undelivered.

**wait-for-mention:** no `wait_for_mention`/`WaitForMention` symbol exists in the tree (grep empty). If such polling exists it would be a hook-driven turn (path A) and thus subject to C2; there is no PTY-independent idle waiter in the code today.

## 3. Detection — can the daemon tell there's no injectable endpoint?

**Yes, at delivery time, but it does nothing with the signal.** The distinguishing signal is the **absence of a `pty_session` alias** for the session: `aliases_for_session` (`src/state/aliases.rs:177`) returns no row of kind `"pty_session"`, so `ring_doorbells` computes `pty_id = None` (`delivery.rs:163-166`) and the reconciler explicitly classifies this as `DeferNoEndpoint` (distinct from `ClearDeadEndpoint`, which is the *stale-alias* case at `model.rs:193-201`). So the code already **names** the "session with pending mail but no injectable endpoint" state — it just drops it on the floor (`mod.rs:238`).

What distinguishes a daemon-owned PTY pane from a bare/iTerm process:
- **Daemon-owned:** has a `pty_session` alias (sourced from `TENEX_EDGE_PTY_SESSION`, `supervisor.rs:51` → `observation.rs:33`), and `crate::pty::is_live(pty_id)` is checkable.
- **Bare/iTerm:** only a `child_pid` (process-tree-derived) and possibly a `harness_session`/`resume` alias; no `pty_session` alias. `child_pid` liveness (`pid_alive`) says the *process* exists but says nothing about injectability. There is **no tty/pane field** recorded, and the daemon never probes the child's controlling tty.

So detection is a one-line store read (does `aliases_for_session` contain a live `pty_session`?), and it is in fact already computed inside the reconciler.

## 4. Feasibility notes for the three planned changes

**(a) Warn in the "you are X" preamble when not PTY-wrapped.**
The preamble is `render_self` — `"\n  You are @{}, running on {}."` in `src/fabric_context/render.rs:75-92`. Warnings already have a first-class channel: `warnings` are collected in `assemble_turn_start` (`src/turn_context/start.rs:63`, pushed e.g. at `:121`) and rendered via `render_warnings` (`render.rs:61`); the not-a-member warning is the existing template to copy. The not-wrapped signal is a cheap store read available right where warnings are assembled: check `aliases_for_session(&rec.session_id)` for a live `pty_session` alias (same predicate the doorbell uses, `delivery.rs:163-167`). Feasible with no new plumbing — push a warning like "this session is not hosted in a daemon PTY; mentions received while you are idle will NOT reach you until you take a turn." Caveat: the warning only lands *when the agent takes a turn* (it rides path A), which is exactly when it's least needed — so it informs the human/agent but doesn't fix C1.

**(b) A `./skills/tenex-edge/` resource.** `skills/tenex-edge/` already exists (`SKILL.md`, `agents/`, `references/`). A reference doc should tell an agent: (i) if your session wasn't launched by the daemon (no `TENEX_EDGE_PTY_SESSION`), you are "online but deaf" — you receive mentions only during turns you initiate; (ii) messages sent to you while idle sit `pending` in the inbox and are never pushed (C1); (iii) `who`/presence will still show you online, misleading others (C4); (iv) the daemon will NOT auto-spawn a reachable copy because your alive row suppresses the offline handler (C3); (v) remedy: re-home the session under a daemon PTY (see 4c) or keep taking turns / poll; (vi) to check: your session lacks a `pty_session` alias.

**(c) A `tenex-edge my session pty-wrap-me` command.**
Surface: the `my session` command currently exposes only `End` (`src/cli/session.rs:4-8`); a new `SessionAction::PtyWrapMe` slots in beside it, self-anchored the same way `end --self` is (`session.rs:35-47` → `pty_session_env()` or `TENEX_EDGE_SESSION`). Ironically, for a non-PTY session `pty_session_env()` is empty, so the anchor must come from `TENEX_EDGE_SESSION`.
The daemon already has the primitive: `resume_agent(state, slug, root, resume_id)` opens the harness inside a fresh daemon PTY supervisor and re-registers it (`src/session_host/launch.rs:209-258`), which will set `TENEX_EDGE_PTY_SESSION` and write the `pty_session` alias. Inputs needed are all on the `sessions` row: `resume_id` (`ddl.rs:154`, read at `sessions.rs:35`), `agent_slug`, `channel_h`. Codex's resume shape is `codex resume <id>` (`src/session_host/registry.rs:83`, `ResumeShape::Subcommand("resume")`), assembled by `build_resume_command` (`registry.rs:103`). A resumed codex/claude re-registers under the same canonical id and re-derives the same pubkey (comment at `launch.rs:239-240`), so identity is preserved.
Risks:
- **In-flight turn / lost work:** killing the current process (`pty::kill`) mid-turn drops any uncommitted work; must gate on `working=0`.
- **Double-spawn / claim race:** the old alive row + the new resumed session could both be `alive=1` transiently; the offline handler's `has_alive`/claim logic (`offline_mention.rs:42-86`) and session-claim ownership must be reconciled, and the old row must be `mark_dead` first, or two PTYs race to inject.
- **Lost scrollback / context:** resume replays the harness's native session (codex `resume <id>`), so terminal scrollback in the iTerm tab is gone; only what the harness persisted comes back — risky for very large contexts (the 104M-token case) where re-hydration is expensive or may exceed limits.
- **cwd/workspace resolution:** resume takes no client cwd and reads the recorded workspace path (`launch.rs:34-40`); if that row is missing it fails loud rather than guessing.

### Key file:line index
- Silent black-hole: `src/reconcile/delivery/model.rs:184-192` (DeferNoEndpoint) + `src/reconcile/delivery/mod.rs:238` (no-op translate)
- Endpoint resolution from alias: `src/session_host/delivery.rs:163-167`
- `pty_session` alias source: `src/pty/supervisor.rs:51`, `src/cli/hooks/observation.rs:33`
- Self-drain: `src/turn_context/reads.rs:32-38`, `src/turn_context/start.rs:151`, `src/turn_context/check.rs:43`
- Offline handler suppressed by alive: `src/daemon/server/demux/offline_mention.rs:42-59`; dispatch site `src/daemon/server/demux.rs:170`
- Liveness = PID only: `src/daemon/server/engine_lifecycle.rs:263-265`; reaper `src/daemon/server/membership_cleanup.rs:82`
- replay_chat gate: `src/reconcile/session_start/mod.rs:290`
- Doorbell injection: `src/session_host/delivery/prompt.rs:80,128`
- Preamble + warnings: `src/fabric_context/render.rs:75-92,61`
- `my session` surface: `src/cli/session.rs`; PTY resume: `src/session_host/launch.rs:209-258`; codex resume shape: `src/session_host/registry.rs:83`

