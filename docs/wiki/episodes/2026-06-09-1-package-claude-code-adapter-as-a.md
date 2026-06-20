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
captured_at: 2026-06-17T23:45:24Z
---

# Episode: Package Claude Code adapter as a plugin, binary stays separate

## Prior State

Claude Code integration was hand-installed: manual merge of hooks into ~/.claude/settings.json, a Python dispatcher at ~/.local/bin/tenex-edge-hook.py, and a skill directory — all fragile, no install/uninstall/update story.

## Trigger

User asked whether tenex-edge should be packaged as a Claude Code plugin too.

## Decision

Yes — package the CC adapter (hooks + tenex-send-message skill + dispatcher) as a proper CC plugin. The Rust binary (substrate) stays a separate install. A SessionStart bootstrap hook gracefully degrades when the binary is absent. The word 'plugin' must not leak into tenex-edge's vocabulary — it's just adapter #3 getting proper packaging.

## Consequences

- The adapter → substrate dependency arrow is preserved (adapter depends on substrate, never reverse).
- CC plugins distribute as git-hosted scripts, not platform binaries — so the Rust binary cannot live inside the plugin repo cross-platform.
- Plugin solves settings.json fragility but NOT the macOS codesign/xattr reinstall gotcha — that lives with the binary.
- Symmetry across three host adapters (Codex config.toml, OpenCode TS plugin, CC plugin) becomes a natural property, not coupling.

## Open Tail

- Plugin manifest/structure not yet sketched — assistant offered, awaiting user go-ahead.
- Persistence architecture choice (single-writer daemon vs per-session DB) will shape what the plugin's bootstrap hook actually starts/connects-to.

## Evidence

- transcript lines 1-91

