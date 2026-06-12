---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: product
status: superseded
subjects:
  - status-distillation
  - edge-distillation-role
  - transcript-reader
supersedes: []
related_claims: []
source_lines:
  - 4014-4019
  - 4191-4193
  - 4279-4279
  - 4410-4428
  - 4518-4520
  - 4665-4698
  - 4766-4781
captured_at: 2026-06-12T19:51:37Z
---

# Episode: Agent status distillation: transcript-first, native rig, ~/.tenex config

## Prior State

Agent status was distilled heuristically from raw tool-call names (e.g. 'Running: find … -name *.toml' or 'Reading util.rs'), not from the agent's conversation. An external Python script calling OpenRouter via a TENEX_EDGE_DISTILL_CMD env var (only in ~/.claude/settings.json) was the only LLM path — which OpenCode and other hosts never inherited, falling back to mechanical tool-name echoing.

## Trigger

Three user corrections: (1) 'it needs to use the conversation transcript, just like pc does — not invent intent from isolated tool names' (line 4191-4193); (2) 'fix opencode… stop taking shortcuts!' (line 4279) when OpenCode was left on heuristic-only; (3) 'the configuration for distill MUST live in ~/.tenex/ including which llm provider and llm model to use… using the existing format for providers.json and models.json… use a role for edge-distillation' (line 4518-4520).

## Decision

Status is now distilled from the conversation transcript (not tool names) via native rig-core LLM calls, configured through ~/.tenex/llms.json (edge-distillation role → named configuration → {provider, model}) and ~/.tenex/providers.json for credentials, supporting openrouter (api_key) and ollama (base_url). All three hosts produce intent-level status: Claude Code and Codex pass transcript_path from their file-based transcripts; OpenCode fetches from the SDK message store → flat JSONL temp file. Resolution ordering: $TENEX_EDGE_DISTILL_CMD override → rig+edge-distillation role → heuristic fallback.

## Consequences

- OpenCode now has full transcript-based distillation parity (SDK message store → temp JSONL → --transcript), not a heuristic shortcut
- The Python distiller script and TENEX_EDGE_DISTILL_CMD in ~/.claude/settings.json were removed; rig+role is the default path
- A 20s distill gate interval applies uniformly across hosts, preventing per-tool-call spam
- transcript.rs read_recent accepts both Claude Code's nested {type, message.content} shape and the flat {role, content} shape the OpenCode plugin writes
- Distiller calls are synchronous in the engine loop (~1-2s pause per gate interval); can be made async later if it matters

## Open Tail

- The forked engine takes ~10-13s after session start before first status appears (relay auth warmup + mention fetch + rig network call) — brief 'idle' at start is expected

## Evidence

- transcript lines 4014-4019
- transcript lines 4191-4193
- transcript lines 4279-4279
- transcript lines 4410-4428
- transcript lines 4518-4520
- transcript lines 4665-4698
- transcript lines 4766-4781

