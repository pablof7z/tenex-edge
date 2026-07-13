---
type: episode-card
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
salience: root-cause
status: superseded
subjects:
  - defer-no-endpoint
  - pty-wrap-me
  - session-kill-self
  - preamble-warning
  - delivery-reconciler
supersedes: []
related_claims: []
source_lines:
  - 679-712
  - 742-773
  - 824-841
  - 843-845
  - 953-984
  - 1059-1069
  - 1180-1194
  - 1231-1271
captured_at: 2026-07-13T07:31:31Z
---

# Episode: Non-PTY session delivery black-hole: diagnosis and three-part remediation

## Prior State

The delivery reconciler had four outcomes for pending mentions: Inject, DeferDebounced, ClearDeadEndpoint, and DeferNoEndpoint. DeferNoEndpoint was intended to mean 'defer until the session has an endpoint,' but its translate() handler was a literal no-op (=> {}). Sessions launched outside a daemon PTY (e.g., a human typing 'codex resume' into a bare iTerm tab) have no pty_session alias, so pty_id resolves to None, triggering DeferNoEndpoint. This caused messages to silently accumulate as pending forever — no inject, no retry, no failure event, no log. The offline-mention recovery path that could re-home such sessions was short-circuited because liveness was pure PID existence (alive=1), so the handler early-returned. Additionally, who/presence falsely showed these sessions as online and deliverable.

## Trigger

User reported that the slate-falcon agent in a NIP-29 group was ignoring messages, and that agents in the tenex-edge workspace were ignoring their p-tags. Investigation revealed the slate-falcon session was alive but running outside a daemon PTY. An Opus agent deep-read the delivery code and confirmed the root cause: DeferNoEndpoint at src/reconcile/delivery/mod.rs:238 is byte-for-byte a no-op, and the offline-mention handler (offline_mention.rs:42-59) early-returns because the session is marked alive. The user then directed: 'send sonnet agents to implement + PR + merge pty-wrap-me, 354, the you are X not PTY-wrapped preamble stuff.'

## Decision

Three fixes were implemented and merged: (1) PR #404 — 'session kill --self' lets a hosted agent terminate its own process via the existing session_kill RPC (--self-only, no positional target), closing #354. (2) PR #405 — a not-PTY-wrapped preamble warning pushed into the turn-context warnings channel (same mechanism as the existing not-a-member warning) when a session lacks a live pty_session alias, informing the agent that idle mentions won't reach it. (3) PR #407 — 'my session pty-wrap-me --self' adds a session_pty_wrap daemon RPC that atomically re-homes a non-PTY session into a daemon PTY: it refuses if already-wrapped/mid-turn/not-resumable, kills the old process via session_kill, then resumes the same harness session via resume_agent under a fresh daemon PTY supervisor, preserving identity (same canonical id + pubkey). All three use --self-only semantics (an agent may only act on its own session).

## Consequences

- Agents in non-PTY sessions now receive a preamble warning on their first turn that idle mentions will not be pushed — though the warning only lands when the agent takes a turn (path A), which is when it's least needed.
- Agents can self-terminate via 'session kill --self' and self-re-home via 'session pty-wrap-me --self', both constrained to --self-only with no positional target.
- The pty-wrap-me RPC is atomic server-side (kill + resume in one call), avoiding a race across CLI round-trips, and marks the old session row dead before resuming to prevent a double-inject/claim race.
- The core DeferNoEndpoint no-op at mod.rs:238 is still unfixed — the three shipped changes are mitigations (warn, self-service re-home) but do not change the reconciler's silent-drop behavior for sessions that remain endpoint-less.
- The pty-wrap-me kill+resume path was NOT exercised end-to-end (needs live relay + harness auth); only its decision logic is unit-covered. It relies on resume_agent/resume_token_for being already used by three other production RPCs. A live smoke test was dispatched to the tenex-edge-dev lab but had not completed by session end.
- Scrollback/context loss risk: pty-wrap-me replays only harness-persisted context (terminal scrollback is gone) — genuinely risky for very large sessions (e.g., a 104M-token case).
- A new ACP transport path was added between diagnosis and remediation, shifting the delivery landscape — but since DeferNoEndpoint still no-ops, endpoint-less sessions black-hole regardless of transport.

## Open Tail

- DeferNoEndpoint still silently no-ops for sessions that never self-re-home — the systemic fix (emit a delivery_failure tail event and/or trigger the offline re-home path from DeferNoEndpoint) was proposed but not implemented.
- Live smoke test of pty-wrap-me against a real agent was in progress via tenex-edge-dev lab at session end.
- Codename format bug identified but not resolved: agents emit 'codex-<name>' (prefixed) handles that fail to resolve to pubkeys, while the roster uses '<name>-codex' (suffixed) — causing agents to reply to themselves and p-tag the wrong identity. This may feed the same substrate as the now-fixed #375 classifier loop.

## Evidence

- transcript lines 679-712
- transcript lines 742-773
- transcript lines 824-841
- transcript lines 843-845
- transcript lines 953-984
- transcript lines 1059-1069
- transcript lines 1180-1194
- transcript lines 1231-1271

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-session-delivery-black-hole.json`](transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-session-delivery-black-hole.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-session-delivery-black-hole.json`](transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-session-delivery-black-hole.json)
