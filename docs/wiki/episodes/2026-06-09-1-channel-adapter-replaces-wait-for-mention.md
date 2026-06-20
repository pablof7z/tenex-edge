---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: reversal
status: active
subjects:
  - tenex-edge-channel-adapter
  - wait-for-mention
  - async-work-injection
supersedes:
  - 2026-06-09-1-agent-mention-reactivity-via-wait-for
related_claims: []
source_lines:
  - 1-5
  - 101-135
  - 706-732
  - 936-951
captured_at: 2026-06-17T23:48:53Z
---

# Episode: Channel adapter replaces wait-for-mention for async work injection

## Prior State

Async work injection used a polling-based wait-for-mention hack: manual re-arm after every wakeup, 5-min timeout lapses silently, one batch then exits, and critically could not wake a cold/idle session (no LLM call at session start, so deaf until first UserPromptSubmit).

## Trigger

User directive pointing to Claude Code channels reference: 'we should be using this for injecting async work (instead of wait-for-mention hack)'. Channel docs confirmed push events wake an open-but-idle session with no human prompt — closing the cold-start gap.

## Decision

Adopt Claude Code's channels MCP server as the primary async-work mechanism. Built integrations/claude-code/channel/ (Bun MCP channel server) with self-re-arming wait-for-mention loop emitting <channel> events and a reply tool. Wired into .mcp.json + ~/.claude.json. wait-for-mention is now historical for the Claude Code host.

## Consequences

- Cold/idle sessions are now wakeable — channel events trigger turns with no human prompt (proven end-to-end in live session)
- Channel adapter points at the installed daemon binary; daemon cutover repoints it automatically
- Cross-harness research found Codex (app-server turn/start) and OpenCode (prompt_async) also support idle-session wake — daemon's planned subscribe --json seam can feed all three adapters
- Reply path initially appeared broken (led to routing bug discovery, fixed separately)

## Open Tail

- Codex and OpenCode channel adapters not yet built; subscribe --json seam is designed but not implemented

## Evidence

- transcript lines 1-5
- transcript lines 101-135
- transcript lines 706-732
- transcript lines 936-951

