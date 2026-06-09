---
title: Tenex-Edge Host Adapter
slug: tenex-edge-host-adapter
topic: tenex-edge
summary: Host adapters must carry no identity logic or fabric logic and must never block the editor on the daemon being healthy (fail open)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:96aedf14-df2c-425b-b548-0fa7d1c1ba63
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:05b89548-666c-4e24-a2f5-8a1e92f0bf04
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
---

# Tenex-Edge Host Adapter

## Design Principles

Host adapters must carry no identity logic or fabric logic and must never block the editor on the daemon being healthy (fail open). The dependency arrow between hosts and tenex-edge points one direction only: hosts depend on tenex-edge, never the reverse; grepping the tenex-edge codebase for host names returns nothing. Tenex-edge has no concept of any specific host (no pc, no Claude Code); it exposes a generic host-agnostic boundary for reporting activity and subscribing to awareness, containing no host-specific names. MCP is the natural shape of this host-agnostic boundary — a standard interface the substrate speaks so that any external component can integrate with zero bilateral knowledge in either direction. The engine itself must NOT be the MCP server, to preserve the host-agnostic boundary and avoid giving Claude Code ownership of the engine lifecycle.

<!-- citations: [^f3a73-95] [^f3a73-13] [^f3a73-51] [^f3a73-105] [^162f9-8] -->
## Host-Specific Adapter Patterns

Activity distillation is turn-bracketed, not tool-driven: hosts call `tenex-edge turn-start --session <sid> [--transcript <path>]` when the agent begins working on a user request and `tenex-edge turn-end --session <sid>` when the agent finishes responding. The engine owns the timer — ~30s after turn-start it LLM-distills a status from the transcript, then re-distills every 5 minutes until turn-end; short turns (finishing under 30s) never trigger an LLM call. Hooks only flip turn-start/turn-end and supply a transcript path; they never distill. The legacy tool-driven `observe` verb and PostToolUse-based distillation have been removed. turn-start/turn-end are no-ops when the session id is unknown.

Channels are used for injecting async work instead of the wait-for-mention hack. The substrate exposes a streaming mention subscription (NDJSON, one mention per line) as the generic seam consumed by channel and future adapters, while wait-for-mention remains the portable floor. The channel adapter lives outside tenex-edge (in integrations/claude-code/) and is a Claude-specific upgrade, not a fabric-wide replacement for wait-for-mention.

All three harness hooks (Claude Code, Codex, OpenCode) inject the `wait-for-mention` instruction and point directly to the source tree rather than deployed copies. The Claude Code adapter (hooks, tenex-send-message skill, and dispatcher) is packaged as a plugin; the tenex-edge Rust binary is a separate install, not bundled inside the plugin. The plugin's SessionStart hook gracefully degrades when the tenex-edge binary is absent from PATH. The term 'plugin' does not leak into tenex-edge's vocabulary; it remains purely an adapter packaging concept — the dependency arrow points adapter → substrate, never reverse. Claude Code integration uses hooks: SessionStart drives tenex-edge session-start, SessionEnd drives tenex-edge session-end + pc capture, UserPromptSubmit drives tenex-edge turn-start plus inbox/who injection plus pc inject, and Stop drives tenex-edge turn-end. The Claude Code config points directly to the source tree at `integrations/claude-code/te-hook.py`, and the previously deployed copy at `~/.local/bin` has been deleted. Pc is reduced to a context-injection straw (inject + capture only); tenex-edge drives awareness and pc's legacy awareness module will be removed. The UserPromptSubmit hook injects the available agents list (who output) into context so the agent automatically knows who's reachable. Codex uses `[[hooks.SessionStart/UserPromptSubmit/Stop]]` in `~/.codex/config.toml` (same event names as Claude Code), trusted via Codex's interactive `/hooks`, but hooks do not fire in `codex exec` (interactive TUI only); the reliable Codex integration is the `tenex-codex` launcher wrapper. Codex has a real push/wake equivalent via codex app-server (JSON-RPC turn/start) rather than MCP channels, but adopting it requires changing how Codex sessions are launched (app-server daemon + --remote TUI attachment). Codex hook payloads carry both `session_id` and a live `transcript_path` (a JSONL rollout file Codex keeps appending to during the turn), so UserPromptSubmit maps to `turn-start --transcript <transcript_path>` and Stop maps to `turn-end`; the engine re-reads that same live file as the turn progresses. Codex's Stop hook signals turn end (not session end) — Codex does not currently document a SessionEnd hook, so the Codex adapter passes the Codex process id to `session-start --watch-pid` and the tenex-edge liveness reaper stops presence when Codex exits. OpenCode uses a TypeScript plugin (`~/.config/opencode/plugin/tenex-edge.ts`) with `experimental.chat.messages.transform` for inject and turn-start, and the `event` handler's `session.idle` event for turn-end, rather than CLI hooks. OpenCode has no transcript file, so the plugin keeps a deterministic temp JSONL snapshot (`tenex-oc-<opencode-session-id>.jsonl`) fresh — rewriting it at turn-start and on each `tool.execute.after` — so the path handed to the engine always reflects the recent conversation. Because `experimental.chat.messages.transform` fires once per model invocation (many times per user turn in an agentic loop), turn-start is gated to fire only when the latest user message id changes, so the engine's distillation timer is not reset on every tool round-trip. The opencode plugin version must match the opencode host version (1.16.2) across all installation directories. The canonical source for the tenex-edge plugin is `~/src/tenex-edge/integrations/opencode/tenex-edge.ts` and for the proactive-context plugin is `~/src/proactive-context/integrations/opencode/`. The plugin code (tenex-edge.ts, proactive-context.ts) is compatible with the 1.16.2 SDK without requiring code changes. When reinstalling the binary on macOS, `cp` leaves a stale signature and `com.apple.provenance` xattr that causes macOS to SIGKILL the binary on the fork/re-exec path; reinstalls must `xattr -cr` and `codesign --force --sign -`.

<!-- citations: [^f3a73-52] [^f3a73-61] [^f3a73-69] [^96aed-1] [^f3a73-96] [^96aed-8] [^f3a73-106] [^95659-2] [^3da7f-4] [^05b89-1] [^162f9-9] -->
