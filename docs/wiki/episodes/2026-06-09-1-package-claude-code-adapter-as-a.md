---
type: episode-card
date: 2026-06-09
session: 05b89548-666c-4e24-a2f5-8a1e92f0bf04
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/05b89548-666c-4e24-a2f5-8a1e92f0bf04.jsonl
salience: architecture
status: active
subjects:
  - claude-code-plugin
  - tenex-edge-adapter-packaging
supersedes: []
related_claims: []
source_lines:
  - 1-91
captured_at: 2026-06-12T19:58:43Z
---

# Episode: Package Claude Code adapter as a plugin, binary separate

## Prior State

The Claude Code adapter was hand-installed via fragile settings.json merge + backup dance, with a Python dispatcher hook, skill directory, and manual PATH setup.

## Trigger

User asked whether tenex-edge should be packaged as a Claude Code plugin.

## Decision

Yes — package the CC adapter (hooks + tenex-send-message skill + dispatcher) as a plugin. The Rust binary (substrate) stays as a separate install (brew/cargo). A SessionStart bootstrap hook checks for the binary on PATH and gracefully degrades if absent. The word 'plugin' must never leak into tenex-edge's vocabulary; this is just adapter #3 getting proper packaging, symmetric with the Codex and OpenCode adapters.

## Consequences

- Settings.json merge fragility eliminated by one-command plugin install/uninstall/update
- Plugin cannot ship the Rust binary cross-platform, so binary remains a separate prerequisite
- Plugin solves packaging but NOT the macOS codesign/xattr reinstall gotcha — that lives with the binary
- The persistence architecture choice (single-writer daemon vs per-session DB) shapes what the plugin's bootstrap hook actually starts/connects-to

## Open Tail

- Plugin manifest/structure sketch not yet produced
- Persistence architecture choice unresolved — blocks final plugin bootstrap design

## Evidence

- transcript lines 1-91

