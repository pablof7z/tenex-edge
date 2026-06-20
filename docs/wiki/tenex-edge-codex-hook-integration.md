---
title: tenex-edge Codex Hook Integration
slug: tenex-edge-codex-hook-integration
topic: tenex-edge
summary: Codex hooks run from `hooks.json` or inline `[hooks]` tables in `config.toml`, including project-local `.codex/config.toml`
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-16
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:rollout-2026-06-09T00-10-41-019ea912-c93f-7c90-a16d-d46484711d29
  - session:rollout-2026-06-09T10-49-43-019eab5b-d8e8-7952-81ba-2fda676ef2d1
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-09T12-58-38-019eabd1-dde2-76c2-84e3-9edc3e78e48f
  - session:rollout-2026-06-16T12-56-00-019ecfdb-f9ce-72b1-b465-398423cae745
  - session:rollout-2026-06-16T14-02-11-019ed018-926e-7c40-bf14-796efbec0b7a
---

# tenex-edge Codex Hook Integration

## Hook Configuration

Codex hooks run from `hooks.json` or inline `[hooks]` tables in `config.toml`, including project-local `.codex/config.toml`. The project-local `.codex/config.toml` hook configuration is removed in favor of the global `~/.codex/config.toml` as the single source for running hooks. The global config includes trusted hook hashes for `SessionStart`, `PostToolUse`, and `UserPromptSubmit` entries so Codex does not require interactive trust approval. The config template includes the `UserPromptSubmit` status message. The prior user config is backed up at `~/.codex/config.toml.tenex-edge-backup-20260609-0010`.

<!-- citations: [^rollo-1] [^rollo-6] -->
## Hook Events and Script

The Codex integration hook script `te-hook.py` delegates to the native Rust command `tenex-edge hook --host codex`. The active hook dispatches to the tenex-edge binary on PATH at /Users/pablofernandez/.local/bin/tenex-edge. It handles `SessionStart`, `PostToolUse`, and `UserPromptSubmit` events. `UserPromptSubmit` hook stdout is added as developer context by Codex. The injected context explicitly instructs Codex to run `send-message` when asked to message a peer, rather than claiming it cannot. Codex documents `SessionStart`, `PostToolUse`, `UserPromptSubmit`, and `Stop` hooks, but not `SessionEnd`. Process watching replaces the nonexistent `SessionEnd` hook for tenex-edge presence cleanup on Codex exit. The `SessionStart` hook is narrowed to `startup|resume` to prevent duplicate tenex-edge engines during compaction. The hook adapter is fail-open: if tenex-edge is unavailable, Codex continues normally. The hook adapter accepts `conversation_id`, `thread_id`, and camelCase session-id field variants in addition to `session_id`. The adapter writes quiet diagnostics to `~/.tenex/edge/codex-hook.log`. The Codex README documents the accepted session-id field variants and the hook log path. The Claude hooks config must use the TENEX_EDGE_BIN environment variable (resolving to ~/.local/bin/tenex-edge) rather than hard-coding the debug binary path, to avoid version-skew risk. The opencode adapter forces `TENEX_EDGE_AGENT=opencode` when calling the hook, rather than inheriting the ambient `TENEX_EDGE_AGENT` environment variable. The Rust hook ignores an inherited `TENEX_EDGE_AGENT` value for the `opencode` host unless an explicit opencode-specific override is provided.

<!-- citations: [^rollo-2] [^rollo-5] [^rollo-15] [^rollo-20] [^rollo-59] [^rollo-69] -->
## Legacy Wrapper Removal

The `tenex-codex` wrapper script is removed from the repository. The stale `tenex-codex` wrapper executable is disabled via a timestamped rename (`tenex-codex.disabled-20260609-010018`) and is no longer in PATH. <!-- [^rollo-3] -->
