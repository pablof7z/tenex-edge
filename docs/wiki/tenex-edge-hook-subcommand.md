---
title: Tenex-Edge Hook Subcommand
slug: tenex-edge-hook-subcommand
topic: tenex-edge
summary: `tenex-edge hook --host <name> --type <hook-type>` is the sole host-facing entry point for session and turn lifecycle operations
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-12
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:9ac666e5-b468-4af2-be5e-83e5c8f2d1d2
  - session:081ec521-c99b-42fb-9aa7-4a109519a62f
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:2cee1bc6-0f1a-4746-9de6-68ca1a7e2737
  - session:f9bdcf4c-c972-46ff-91b8-9e30785d3331
  - session:rollout-2026-06-09T00-10-41-019ea912-c93f-7c90-a16d-d46484711d29
  - session:rollout-2026-06-09T10-49-43-019eab5b-d8e8-7952-81ba-2fda676ef2d1
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-09T12-58-38-019eabd1-dde2-76c2-84e3-9edc3e78e48f
---

# Tenex-Edge Hook Subcommand

## Hook Subcommand Role

`tenex-edge hook --host <name> --type <hook-type>` is the sole host-facing entry point for session and turn lifecycle operations. It dispatches to internal private functions while adding stdin JSON parsing, session-id extraction, PID handling, and output-format selection. The `user-prompt-submit` hook type does more than the bare `turn-start` verb by also publishing the prompt as a kind:1 OP, making `hook` the real surface for that behavior. The opencode integration was migrated from calling bare verbs directly to piping a JSON payload to `tenex-edge hook` and reading stdout back; it passes an explicit `pid` field in its hook JSON payload rather than relying on ancestor pid_search. All three harness hook configurations point directly to the source tree (not to copies in `~/.local/bin`), so that local edits are live immediately without manual redeployment. The Python wrapper approach for hooks is abandoned; all integrations use the new shape invoking the binary directly (`tenex-edge hook --host <host> --type <hook-type>`). The tenex-edge integration on the remote machine is configured via hooks in ~/.claude/settings.json calling the Rust binary directly, not via an MCP server. Claude Code's installed settings include tenex-edge hooks alongside existing pc hooks: SessionStart (`tenex-edge hook --host claude-code --type session-start`), UserPromptSubmit (tenex-edge runs first then pc inject), Stop (tenex-edge runs first then pc capture --in 300), SessionEnd (tenex-edge runs first then pc capture). The global ~/.claude/settings.json is updated to remove the dead PostToolUse→observe wiring and add Stop→turn-end. The `hook` subcommand reads stdin JSON, extracts fields per the host's field list, and dispatches to existing command functions — it never needs host-specific conditional logic. The hook architecture is well-structured to avoid complexity when supporting 50 agent harnesses.

<!-- citations: [^2cee1-17] [^2cee1-19] [^081ec-1] [^081ec-2] [^9ac66-1] [^9ac66-8] [^3da7f-12] [^95659-9] [^f9bdc-3] -->
## Standalone Verb Status

The standalone verbs `session-start`, `session-end`, `turn-start`, `turn-check`, and `turn-end` are removed from the public CLI surface; they exist only as private functions called internally by `tenex-edge hook`. (Previously: they served as the callable core wrapped by hook and as manual/debug entry points.)

<!-- citations: [^9ac66-2] [^9ac66-9] -->
## Commands Outside Hook Scope

The commands `who`, `tail`, `doctor`, `project`, `acl`, and `send-message` are not superseded by `hook` because they are interactive queries, owner config, or agent-initiated actions. The `inbox`, `who`, `send-message`, and `wait-for-mention` commands remain on the CLI as manual/agent-facing commands. The `inbox` command specifically serves the opencode injection path and manual message-checking use; Claude Code and Codex drain via the hook path. OpenCode uses a TypeScript plugin (~/.config/opencode/plugin/tenex-edge.ts) with experimental.chat.messages.transform for injection and tool.execute.after for observation. Codex uses the same hook event names as Claude Code (SessionStart, UserPromptSubmit, PostToolUse, Stop) configured in ~/.codex/config.toml. The Codex config template (config.template.toml) includes the `UserPromptSubmit` status message alongside `SessionStart` and `Stop`. Multiple hook sources in Codex all run, and hooks are enabled by default globally. Non-managed hooks in Codex are trusted by exact hook hash stored in `[hooks.state]`. Codex integration uses direct binary invocation (`tenex-edge hook --host codex --type <hook-type>`) without Python wrapper references or `__HOOK__` substitution steps. The active Codex hook binary is installed at /Users/pablofernandez/.local/bin/tenex-edge and dispatches to tenex-edge on PATH. The Codex hook adapter (te-hook.py) delegates to `tenex-edge hook --host codex` rather than stitching context itself, and it accepts multiple session-id field variants including `session_id`, `conversation_id`, and `thread_id` (including camelCase variants) so that the `session-start` hook registers the agent even when Codex sends an alternate field name. The Codex README.md documents the accepted session-id fields and the hook log path. Codex and OpenCode integration hooks are researched and configured by a background agent. The Codex SessionStart hook outputs JSON with a `systemMessage` field containing the wait-for-mention instruction, not plain text. All Codex hook output types share the same base JSON schema fields: `systemMessage`, `suppressOutput`, `stopReason`, and `hookSpecificOutput`. The PostToolUse hook is wired for Codex but not for Claude Code, because Claude Code's PostToolUse stdout format is unverified (may need `additionalContext` JSON rather than plain text).

<!-- citations: [^2cee1-15] [^2cee1-16] [^2cee1-18] [^f3a73-116] [^f3a73-117] [^9ac66-3] [^9ac66-11] [^95659-8] [^f9bdc-4] [^rollo-1] [^rollo-3] [^rollo-9] [^rollo-15] -->
## Help Text Updates

The `--help` descriptions for `session-start`, `session-end`, `turn-start`, `turn-check`, and `turn-end` should be updated to reflect that they are private internal functions invoked by `hook`, removing any implication that harnesses or users call them directly as CLI commands.

<!-- citations: [^9ac66-4] [^9ac66-10] -->

## SID Generation and HostDef Flags

The `hook` session-start path generates and prints a new SID to stdout when the opencode HostDef is used with an empty session id. SID generation on empty session id is gated behind a `generates_sid` HostDef flag, restricted to opencode; for Claude Code and Codex, an empty id remains a fail-open no-op to prevent spawning orphan sessions. <!-- [^9ac66-12] -->

## Testing

The daemon integration smoke test drives the `hook` path via stdin and exercises the generate-and-print-SID branch. <!-- [^9ac66-13] -->
