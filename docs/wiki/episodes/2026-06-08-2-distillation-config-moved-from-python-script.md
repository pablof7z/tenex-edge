---
type: episode-card
date: 2026-06-08
session: f3a730bf-9a3b-4952-b687-c93ade5fd7ec
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f3a730bf-9a3b-4952-b687-c93ade5fd7ec.jsonl
salience: architecture
status: active
subjects:
  - distillation-config
  - edge-distillation-role
  - rig-integration
supersedes: []
related_claims: []
source_lines:
  - 4518-4861
captured_at: 2026-06-17T23:38:11Z
---

# Episode: Distillation config moved from Python script + env var to ~/.tenex/ with native rig

## Prior State

The distiller was a standalone Python script (~/.local/bin/tenex-edge-distill.py) invoked via TENEX_EDGE_DISTILL_CMD env var in ~/.claude/settings.json. Only Claude Code sessions inherited that env var, so OpenCode (launched from the terminal) never saw it and fell back to the heuristic — an environment-scoping bug that made status mechanical on non-Claude hosts.

## Trigger

User directive: 'the configuration for distill MUST live in ~/.tenex/ including which llm provider and llm model to use (with support for ollama and openrouter via rig.rs right now). It should use the existing format for both providers.json and models.json and use a role for edge-distillation to choose the named model.'

## Decision

Distillation config is resolved from ~/.tenex/llms.json (edge-distillation role → named configurations entry → {provider, model}) + ~/.tenex/providers.json (creds). The LLM call is native rig-core 0.37 (openrouter: api_key; ollama: apiKey = base_url), not a Python subprocess. Resolution chain: $TENEX_EDGE_DISTILL_CMD override → edge-distillation role via rig → heuristic fallback. The Python script and settings.json env var were removed.

## Consequences

- Any host (Claude Code, Codex, OpenCode) automatically gets LLM distillation without per-host env configuration — the ~/.tenex/ path is universal.
- No external Python dependency for distillation; the binary handles it natively.
- Adding rig-core brought reqwest → rustls with both ring and aws-lc-rs, causing a CryptoProvider panic on any TLS handshake. Fix: explicitly install ring as the default provider in main() — this must be repeated for any future dep that brings rustls.
- The distillation gate interval (20s when LLM available, 5s heuristic-only) now keys off whether rig resolves a model, not just the env var.
- llmconfig.rs resolves the role via a pure, env-free function (testable with temp dirs without TENEX_DIR races).

## Open Tail

- Adding more providers (anthropic direct, etc.) to llmconfig.rs requires extending the match in resolve_role_in.
- The ~10–13s startup delay before first status publishes (relay auth warmup + mention fetch + rig network call) is expected but may need attention if perceived as broken.

## Evidence

- transcript lines 4518-4861

