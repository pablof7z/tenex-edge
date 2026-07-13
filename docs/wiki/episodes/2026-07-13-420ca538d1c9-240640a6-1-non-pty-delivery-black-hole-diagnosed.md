---
type: episode-card
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
salience: root-cause
status: active
subjects:
  - defer-no-endpoint
  - pty-wrap-me
  - session-kill-self
  - preamble-warning
  - delivery-model
supersedes:
  - 2026-07-13-420ca538d1c9-240640a6-1-non-pty-idle-session-black-hole
related_claims: []
source_lines:
  - 756-773
  - 824-841
  - 940-953
  - 1180-1194
  - 1546-1612
captured_at: 2026-07-13T08:15:49Z
---

# Episode: Non-PTY delivery black-hole diagnosed and mitigated with self-service re-home tooling

## Prior State

Sessions launched outside a daemon PTY (e.g. raw `claude --resume` in an iTerm tab) have no `pty_session` alias. The delivery reconciler's `DeferNoEndpoint` action handles this case with `=> {}` — a silent no-op with no retry, no failure event, and no re-fire. Idle mentions addressed to such a session are permanently lost (black-holed). Agents have no visibility into this condition and no self-service remedy.

## Trigger

User reported that slate-falcon agent was ignoring messages in a NIP-29 group. Investigation traced the delivery path: `decide()` returns `DeferNoEndpoint` when `pty_id == None`, and `translate()` maps it to `=> {}` (mod.rs:238). The agent was alive but its session had no live PTY endpoint, so every idle mention silently accumulated in the inbox with `delivered_at=0`. The root cause is distinct from #375 (management classifier loop) — it is a structural gap in the delivery model for non-PTY sessions.

## Decision

Three self-service mitigations shipped and merged: (1) `session kill --self` (PR #404) — lets a hosted agent terminate its own process; (2) not-PTY-wrapped preamble warning (PR #405) — pushes a warning into the agent's first-turn context when no live `pty_session` alias exists; (3) `session pty-wrap-me --self` (PR #407) — a `session_pty_wrap` daemon RPC that atomically kills the old process and resumes the same harness session into a fresh daemon PTY via `resume_agent`, gated on `working=0` and refusing if already wrapped or not resumable. All three were live-verified end-to-end against a real Claude agent, real host auth, and a real croissant relay.

## Consequences

- Non-PTY agents now receive a first-turn warning informing them they are not PTY-wrapped and idle messages won't reach them.
- Agents can self-rehome into a daemon PTY via `tenex-edge my session pty-wrap-me --self`, preserving the same session ID and pubkey.
- After re-home, previously stuck inbox messages flip from `pending` to `injected` and deliver to the new PTY endpoint.
- The `SessionAction` enum gained two new variants (`Kill`, `PtyWrapMe`), both `--self`-only with no positional target accepted.
- The pty-wrap-me kill+resume path was unit-tested but not exercised end-to-end during initial implementation; the subsequent live smoke test closed that gap (PASS on all 5 steps).
- The #405 warning was corrected by the #407 agent to reference the actual command shape `tenex-edge my session pty-wrap-me --self`.

## Open Tail

- The systemic fix — making `DeferNoEndpoint` itself stop silently no-op'ing (emit a `delivery_failure` tail event and/or trigger the offline re-home path) — was explicitly scoped out and not filed as an issue. `mod.rs:238` still reads `DeferNoEndpoint => {}`.
- A whole ACP transport path was added since the diagnosis, shifting the delivery landscape — but `DeferNoEndpoint` still no-ops regardless of transport.

## Evidence

- transcript lines 756-773
- transcript lines 824-841
- transcript lines 940-953
- transcript lines 1180-1194
- transcript lines 1546-1612

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-delivery-black-hole-diagnosed.json`](transcripts/2026-07-13-420ca538d1c9-240640a6-1-non-pty-delivery-black-hole-diagnosed.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-delivery-black-hole-diagnosed.json`](transcripts/raw/2026-07-13-420ca538d1c9-240640a6-1-non-pty-delivery-black-hole-diagnosed.json)
