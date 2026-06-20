---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: product
status: superseded
subjects:
  - agent-status-distillation
  - transcript-based-intent
supersedes: []
related_claims: []
source_lines:
  - 4191-4265
captured_at: 2026-06-17T23:38:11Z
---

# Episode: Agent status distilled from conversation transcript, not tool names

## Prior State

Agent status was derived from raw PostToolUse tool names via a heuristic echo — e.g. 'Running: find …' or 'Reading util.rs' — producing mechanical, leaky output that revealed shell commands instead of intent.

## Trigger

User correction: 'it needs to use the conversation transcript, just like pc does — this is literally supposed to work exactly the same as pc.' The initial implementation only fed tool names to the LLM, which produced shallow summaries of tool targets rather than intent-level descriptions.

## Decision

Status is now distilled from the recent conversation transcript (last ~14 turns of user prompts + assistant text + tool uses, filtering tool-result noise), exactly like proactive-context. The LLM receives the actual conversation and returns a one-line intent description. The heuristic over raw tool names is retained only as a fallback (no transcript / no LLM).

## Consequences

- All three hosts read the transcript: Claude Code and Codex via transcript_path from hook JSON; OpenCode via client.session.messages → flattened {role,content} JSONL (filtering out injected _tenexInjected peer-briefing parts so they don't pollute the summary).
- The heuristic echo ('Running: find …') is no longer the primary path — it only fires if both transcript and LLM are unavailable.
- A 20s gate interval prevents the LLM from being called on every tool burst.
- The distiller must NOT be the claude CLI (would re-fire hooks recursively) — direct API calls only.
- transcript.rs read_recent accepts both shapes: Claude Code's nested {type, message.content} and the flat {role, content} the OpenCode plugin writes.

## Open Tail

- Other host integrations (mobile apps, etc.) will need their own transcript extraction, mirroring the OpenCode SDK-message-store pattern.

## Evidence

- transcript lines 4191-4265

