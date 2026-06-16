---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: product
status: active
subjects:
  - tenex-edge-distillation
  - agent-status
supersedes: []
related_claims: []
source_lines:
  - 4191-4277
  - 4279-4407
captured_at: 2026-06-16T15:17:45Z
---

# Episode: Distillation input source: transcript, not tool names

## Prior State

Agent status distillation consumed only PostToolUse tool names and targets (e.g. 'Running: find …', 'Reading util.rs'), producing mechanical labels that leaked raw commands to the relay. This matched no other system's behavior.

## Trigger

User correction at line 4191: 'it needs to use the conversation transcript, just like `pc` does — this is literally supposed to work exactly the same as pc'

## Decision

All hosts now distill from the real conversation transcript (the agent's dialog), not from isolated tool events. The heuristic over tool names is a fallback only (no transcript or no LLM available). OpenCode fetches from its SDK message store → temp JSONL (like pc's plugin); Claude Code and Codex pass transcript_path from their hooks.

## Consequences

- Added src/transcript.rs with parsers for both Claude Code's nested {type, message.content} shape and the flat {role, content} JSONL shape that OpenCode/pc write
- PostToolUse hooks now pass --transcript to observe; the engine reads the recent transcript tail (~14 turns) and feeds that to the LLM
- OpenCode plugin filters out its own _tenexInjected peer-briefing parts so they don't pollute the distiller input
- Verified end-to-end: a transcript about a rate-limiter bug but a generic tool target (util.rs) produces 'checking the token-bucket refill calculation' — phrase only present in the conversation

## Open Tail

- OpenCode transcript fetch distinguishes the opencode session ID (for message fetch) from the tenex-edge session ID (for --session)

## Evidence

- transcript lines 4191-4277
- transcript lines 4279-4407

