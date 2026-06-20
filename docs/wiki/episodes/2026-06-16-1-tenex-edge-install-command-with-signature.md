---
type: episode-card
date: 2026-06-16
session: 404ab754-a2d5-4820-a800-6de4972c549c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/404ab754-a2d5-4820-a800-6de4972c549c.jsonl
salience: product
status: active
subjects:
  - tenex-edge-install
  - hook-deduplication
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 310-315
  - 417-428
  - 485-494
captured_at: 2026-06-18T00:36:54Z
---

# Episode: tenex-edge install command with signature-based hook dedup

## Prior State

tenex-edge had no install command; hook setup into agent harnesses was manual or ad-hoc, with no standardised wiring or dedup logic

## Trigger

user directive: 'tenex-edge needs an install command just like ../proactive-context's to setup hooks in the different harnesses'

## Decision

Added `tenex-edge install` subcommand mirroring proactive-context's interface (--harness, --all, --dry-run, --status, --uninstall), with three harness strategies: JSON merge for Claude Code (4 hooks + statusLine) and Codex (3 hooks), and file-drop for opencode (embedded plugin via include_str!). Dedup is by hook signature (`hook --host X --type Y`) rather than binary path prefix, so re-installing after a path change (cargo install → dev build) replaces instead of accumulating entries.

## Consequences

- Three harness integrations fully wired from a single command
- Reinstall is idempotent: signature-based matching replaces old entries regardless of binary path changes
- statusLine entry in Claude Code settings is also deduped on reinstall
- opencode plugin is embedded at compile time via include_str!, so install drops a self-contained file

## Open Tail

*(none)*

## Evidence

- transcript lines 1-1
- transcript lines 310-315
- transcript lines 417-428
- transcript lines 485-494

