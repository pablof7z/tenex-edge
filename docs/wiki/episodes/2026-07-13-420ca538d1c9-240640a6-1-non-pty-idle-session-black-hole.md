---
type: episode-card
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
salience: root-cause
status: superseded
subjects:
  - defer-no-endpoint
  - delivery-reconciler
  - pty-wrap-me
  - session-kill-self
  - preamble-warning
supersedes:
  - 2026-07-13-420ca538d1c9-240640a6-1-non-pty-session-delivery-black-hole
related_claims: []
source_lines:
  - 35-53
  - 742-763
  - 824-841
  - 953-984
  - 1059-1070
  - 1180-1271
  - 1546-1612
captured_at: 2026-07-13T08:04:11Z
---

# Episode: Non-PTY idle session black-hole: diagnosis and self-service remediation

## Prior State

Sessions launched outside a daemon PTY (e.g. human-started `codex resume` in iTerm) were believed to be reachable. The delivery reconciler had a `DeferNoEndpoint` variant for sessions with no PTY endpoint, intended to 'defer until an endpoint exists,' but nothing re-fired the decision and no retry was scheduled.

## Trigger

User noticed agents ignoring their p-tags and a slate-falcon agent ignoring messages. Investigation via an Opus agent reading source code revealed that `DeferNoEndpoint` maps to a literal no-op (`=> {}` at `mod.rs:238`), creating a permanent silent black-hole: no inject, no retry, no failure event, no log. Compounding factors: liveness is PID-only so `who` shows the session online (C4); the offline-mention handler that would re-home the agent early-returns because `alive=1` (C3); and the state is never surfaced to any human (C8).

## Decision

Shipped three self-service remediation features via async agents: (1) `tenex-edge my session kill --self` (PR #404, `97322d24`) — lets a hosted agent terminate its own PTY process; (2) preamble warning (PR #405, `ff9ea35d`) — pushes a 'not hosted in a daemon PTY' warning into the agent's turn context when no live `pty_session` alias exists; (3) `tenex-edge my session pty-wrap-me --self` (PR #407, `45e18b63`) — re-homes a non-PTY session into a daemon PTY via a server-side `session_pty_wrap` RPC (refusal checks → kill → resume atomically). The systemic fix (making `DeferNoEndpoint` itself emit a delivery_failure event or trigger auto-rehome) was deliberately scoped out.

## Consequences

- Agents in non-PTY sessions now have a self-service escape hatch (`pty-wrap-me`) to re-home into a daemon PTY without losing their session identity or resume token.
- Agents are warned on their first turn that idle mentions won't reach them if they lack a daemon PTY — though this warning only lands when the agent takes a turn, exactly when it's least needed.
- `session kill --self` and `pty-wrap-me` both add variants to the `SessionAction` enum; future session subcommands must coordinate rebases.
- The live smoke test confirmed the full path: non-PTY agent → black-hole confirmed → pty-wrap-me → same session/pubkey re-homed → stuck mention delivered (`injected, delivered_at>0`).
- `DeferNoEndpoint` still no-ops in the delivery reconciler — the black-hole remains for any session that doesn't self-remediate.
- The pty-wrap-me kill+resume path was not exercised against a live multi-process scenario in the initial PR (only unit-covered); the live lab smoke test subsequently validated it end-to-end.

## Open Tail

- Systemic fix not yet filed: make `DeferNoEndpoint` emit a `delivery_failure` tail event and/or trigger the offline re-home path, so the black-hole is closed for every non-PTY session rather than only those that self-remediate.
- No issue filed for the DeferNoEndpoint no-op defect as of end of session.

## Evidence

- transcript lines 35-53
- transcript lines 742-763
- transcript lines 824-841
- transcript lines 953-984
- transcript lines 1059-1070
- transcript lines 1180-1271
- transcript lines 1546-1612

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-idle-session-black-hole.json`](transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-idle-session-black-hole.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-idle-session-black-hole.json`](transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-idle-session-black-hole.json)
