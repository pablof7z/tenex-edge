---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge
  - claude-channel-adapter
  - async-wake-mechanism
supersedes: []
related_claims: []
source_lines:
  - 1-4
  - 120-133
  - 710-711
  - 793-811
  - 940-951
captured_at: 2026-06-12T20:02:14Z
---

# Episode: MCP channel adapter replaces wait-for-mention polling hack

## Prior State

wait-for-mention mechanism: a CLI verb that polls the inbox in a background loop with a 5-min timeout, must be manually re-armed after each wake, and critically cannot arm at session start (no LLM call happens there) — so a freshly-launched, never-prompted idle agent is deaf to mentions until its human types something.

## Trigger

User pointed to Claude Code channels reference as the proper mechanism for injecting async work, explicitly calling wait-for-mention a 'hack'.

## Decision

Build a Bun MCP channel server (integrations/claude-code/channel/) that pushes inbound mentions as <channel> events into idle sessions, replacing the wait-for-mention re-arm loop. Channel is wired into .mcp.json and enabled in ~/.claude.json.

## Consequences

- Idle sessions can be woken by an inbound event with no human prompt — closes the cold-start deafness gap.
- No more 5-min timeout silent lapses; events queue in order with multi-event support.
- Cross-harness wake research confirmed: Claude=channels, Codex=app-server turn/start, OpenCode=prompt_async — all can wake idle sessions; daemon's subscribe --json seam will feed all three adapters.
- Channel currently requires --dangerously-load-development-channels launch flag (not yet production default).

## Open Tail

- Channel reply-path was proven working but sibling-session delivery required a separate routing fix (see session-aware routing card).

## Evidence

- transcript lines 1-4
- transcript lines 120-133
- transcript lines 710-711
- transcript lines 793-811
- transcript lines 940-951

