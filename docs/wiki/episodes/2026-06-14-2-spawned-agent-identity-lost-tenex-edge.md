---
type: episode-card
date: 2026-06-14
session: d683a556-03b8-4827-b84d-5395cd3610af
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d683a556-03b8-4827-b84d-5395cd3610af.jsonl
salience: root-cause
status: active
subjects:
  - spawn-agent-identity
  - tmux-env-propagation
  - agent-slug-resolution
supersedes: []
related_claims: []
source_lines:
  - 861-865
  - 1062-1129
  - 1076-1080
  - 1161-1193
captured_at: 2026-06-18T00:22:02Z
---

# Episode: Spawned agent identity lost: TENEX_EDGE_AGENT not propagated to tmux pane

## Prior State

spawn_agent passes TENEX_EDGE_AGENT via .env() on the tmux client process. tmux builds a new pane's environment from the server's env plus -e overrides — a variable set only on the client process is silently dropped. TENEX_EDGE_SPAWNED=1 was correctly passed via -e, so the inconsistency was hidden. The session-start hook (hooks.rs:112) resolves slug as TENEX_EDGE_AGENT or else the harness's self-reported default, so the spawn's authoritative identity was lost and the session registered under the harness default (claude).

## Trigger

User spawned a codex agent via tenex-edge, but it appeared as 'claude' in who and tmux display. DB query confirmed all recent sessions have agent_slug=claude despite codex spawns.

## Decision

Move TENEX_EDGE_AGENT from client .env() to the -e flag (which tmux propagates into the pane), and drop the ineffective .env() call. The hook prefers TENEX_EDGE_AGENT over the harness default, making the spawn's known identity authoritative regardless of what the harness self-reports.

## Consequences

- Spawned agents now correctly receive their authoritative slug (e.g. codex, reviewer) instead of falling back to the harness default (claude)
- A custom agent whose slug differs from its launch binary (e.g. 'reviewer' running 'claude') will register under the correct name
- Requires daemon rebuild + restart to take effect

## Open Tail

- Daemon restart pending (other live sessions on the machine)
- This is a separate change from the issue #1 publish fix — should be a distinct commit

## Evidence

- transcript lines 861-865
- transcript lines 1062-1129
- transcript lines 1076-1080
- transcript lines 1161-1193

