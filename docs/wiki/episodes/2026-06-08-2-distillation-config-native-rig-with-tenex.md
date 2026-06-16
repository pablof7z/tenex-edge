---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: architecture
status: active
subjects:
  - tenex-edge-distillation
  - tenex-config
  - llm-integration
supersedes: []
related_claims: []
source_lines:
  - 4518-4520
  - 4643-4700
  - 4700-4780
  - 4783-4860
captured_at: 2026-06-16T15:17:45Z
---

# Episode: Distillation config: native rig with ~/.tenex/ role, not Python script + env var

## Prior State

The LLM distiller was a Python script (~/.local/bin/tenex-edge-distill.py) calling OpenRouter directly, configured via TENEX_EDGE_DISTILL_CMD env var in ~/.claude/settings.json. This env var was only visible to Claude Code sessions — OpenCode (launched from terminal) never inherited it, so it fell back to the mechanical heuristic. An intermediate fix hardcoded the script path as a default, but that was still Python-only and OpenRouter-only.

## Trigger

User directive at line 4518: 'the configuration for distill MUST live in ~/.tenex/ including which llm provider and llm model to use (with support for ollama and openrouter via rig.rs right now)' and 'it should use the existing format for both providers.json and models.json and use a role for edge-distillation'

## Decision

Distillation config lives in ~/.tenex/llms.json under an 'edge-distillation' role key → a named configurations entry {provider, model} → creds from ~/.tenex/providers.json. LLM calls are made natively via rig-core 0.37 (openrouter and ollama providers). The Python script and TENEX_EDGE_DISTILL_CMD env var were removed entirely. Resolution order: $TENEX_EDGE_DISTILL_CMD override → edge-distillation role (rig) → heuristic fallback.

## Consequences

- Added rig-core 0.37 dependency (Cargo.toml alias rig = rig-core), which pulled reqwest with both ring and aws-lc-rs crypto providers, causing a rustls CryptoProvider panic — fixed by explicit ring::default_provider().install_default() in main()
- Added src/llmconfig.rs: resolves role → configuration → {provider, model, api_key/base_url}; supports openrouter (api_key string or array) and ollama (apiKey = base_url); 5 unit tests
- src/distill.rs: added async summarize_via_rig, made distill_activity async; CommandDistiller::resolve() no longer defaults to the Python script path
- ~/.claude/settings.json: TENEX_EDGE_DISTILL_CMD removed; ~/.tenex/llms.json: 'edge-distillation': 'openrouter/openai/gpt-4o-mini' added
- The env-scoping bug (OpenCode couldn't see TENEX_EDGE_DISTILL_CMD from ~/.claude/settings.json) is eliminated — all hosts now get LLM distillation by default through ~/.tenex/
- Old ~/.local/bin/tenex-edge-distill.py remains on disk but is no longer referenced by any code path

## Open Tail

- Only openrouter and ollama providers are supported in llmconfig.rs; other providers in providers.json return None (heuristic fallback)
- Forked engine takes ~10-13s after session start (relay auth warmup + mention fetch + rig network call) before first status appears; brief 'idle' at session start is expected, not a bug

## Evidence

- transcript lines 4518-4520
- transcript lines 4643-4700
- transcript lines 4700-4780
- transcript lines 4783-4860

