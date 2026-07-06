---
type: episode-card
date: 2026-07-03
session: 026d1502-e769-4a9c-9ab5-26169a4150ef
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/026d1502-e769-4a9c-9ab5-26169a4150ef.jsonl
salience: product
status: active
subjects:
  - tenex-edge-config
  - llm-config-management
  - inquire-tui
supersedes: []
related_claims: []
source_lines:
  - 1-91
  - 87-89
  - 91-93
  - 347-398
  - 596-645
  - 647-693
  - 977-977
  - 1345-1359
captured_at: 2026-07-03T11:12:17Z
---

# Episode: tenex-edge config: interactive model provider/role configuration command

## Prior State

providers.json and llms.json (which drive model resolution per role, e.g. edge-distillation) lived under ~/.tenex-edge and were read-only from tenex-edge's side. Users had to edit them by hand. No CLI subcommand or TUI existed for managing them; the only TUI in the codebase was the session-spawner picker (pty_cli/tui_*.rs), unrelated to model config.

## Trigger

User asked if a TUI existed for model configuration, confirmed there wasn't, then directed: use the inquire crate, name it `tenex-edge config`, with provider configuration and live model listing from configured endpoints (Ollama, OpenRouter) with fuzzy search and high-quality UX. A Fable UX agent was spawned in parallel to produce a design spec.

## Decision

Built a new `tenex-edge config` subcommand (8-file module under src/cli/config/, ~1000 LOC) using the inquire crate (v0.9.4, pinned to share the existing crossterm 0.28). Two interactive screens: Providers (add/edit/remove with masked secrets, connection testing, dependent-role warnings) and Models (role picker → provider picker → live fuzzy-searchable model catalog fetched from Ollama /api/tags or OpenRouter /api/v1/models → confirm → atomic write). Unknown JSON keys are round-tripped to preserve compatibility with the wider TENEX format. A Fable UX spec informed the color palette, merged provider-list+actions pattern, error-recovery-in-place, and confirm-before-write semantics.

## Consequences

- providers.json and llms.json are now user-manageable via an interactive CLI flow instead of manual editing only
- New inquire v0.9.4 dependency added, constrained to share crossterm 0.28 (no duplicate crossterm versions)
- CLI dispatch extended with Cmd::Config variant; new args.rs ConfigAction enum (Providers/Models)
- store.rs reads/writes config files preserving unknown keys, with unit tests
- catalog.rs provides live model listing from Ollama and OpenRouter endpoints
- Smoke-tested against a real local Ollama instance: fetched 11 models, fuzzy-searched, wrote edge-distillation → ollama/glm-5.2:cloud to llms.json
- Full 389-test suite and clippy pass clean with no regressions
- Deferred from spec: OSC-11 dark/light theme auto-detection, confirm-screen `was: <old value>` diff line, braille spinner, OpenRouter response caching within a session

## Open Tail

- OSC-11 background luminance detection (terminal-light crate) for automatic dark/light theme switching
- Confirm-before-write screen does not yet show the `was: <old value>` diff line from the spec
- OpenRouter's 300+ model fetch is not cached within a session
- Spinner is a plain dimmed line rather than the braille animation from the spec

## Evidence

- transcript lines 1-91
- transcript lines 87-89
- transcript lines 91-93
- transcript lines 347-398
- transcript lines 596-645
- transcript lines 647-693
- transcript lines 977-977
- transcript lines 1345-1359

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-03-1-tenex-edge-config-interactive-model-provider.json`](transcripts/2026-07-03-1-tenex-edge-config-interactive-model-provider.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-03-1-tenex-edge-config-interactive-model-provider.json`](transcripts/raw/2026-07-03-1-tenex-edge-config-interactive-model-provider.json)
