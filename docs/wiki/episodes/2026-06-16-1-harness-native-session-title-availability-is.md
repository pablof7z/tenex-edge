---
type: episode-card
date: 2026-06-16
session: 68c8bd16-c1bf-4f4a-aed1-89fba263d57d
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/68c8bd16-c1bf-4f4a-aed1-89fba263d57d.jsonl
salience: architecture
status: active
subjects:
  - session-title
  - harness-metadata
  - claude-code
  - codex
  - opencode
supersedes: []
related_claims: []
source_lines:
  - 1-304
captured_at: 2026-06-16T09:36:55Z
---

# Episode: Harness-native session title availability is asymmetric across agents

## Prior State

Unknown whether any of the three agent harnesses expose a native session title field usable by tenex-edge; implicit assumption that availability might be uniform or absent

## Trigger

User asked whether Claude Code, Codex, and OpenCode expose session titles in a way tenex-edge can read

## Decision

Codex and OpenCode both expose native titles (Codex via `thread_name` in `session_index.jsonl`; OpenCode via the `Session.title` field in its SDK). Claude Code has no native title — only `firstPrompt` in `sessions-index.json` and no title in its JSONL transcripts.

## Consequences

- For Claude Code, LLM distillation from the transcript remains the only viable path to produce a session title
- Codex titles can be read via a file lookup keyed by session ID
- OpenCode titles are already available in the live Session object the plugin holds
- Native harness titles represent the model's own summary framing, which may differ in purpose from tenex-edge's 'what is this agent doing right now' distillation — so they serve different use cases even when available

## Open Tail

- Whether tenex-edge should consume native titles from Codex/OpenCode or always distill its own
- How to handle title consistency if native title and distilled title diverge

## Evidence

- transcript lines 1-304

