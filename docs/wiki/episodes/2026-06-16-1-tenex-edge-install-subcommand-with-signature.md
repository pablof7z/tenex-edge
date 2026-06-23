---
type: episode-card
date: 2026-06-16
session: 404ab754-a2d5-4820-a800-6de4972c549c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/404ab754-a2d5-4820-a800-6de4972c549c.jsonl
salience: product
status: active
subjects:
  - tenex-edge-install
  - tenex-edge-hook-dedup
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 310-314
  - 417-428
  - 485-494
captured_at: 2026-06-16T08:15:04Z
---

# Episode: tenex-edge install subcommand with signature-based hook dedup

## Prior State

tenex-edge had no install command; users manually wired hooks into each harness config. proactive-context already had `pc install` with interactive selector, sentinel-based JSON merge, and per-harness strategies.

## Trigger

User explicitly requested tenex-edge match proactive-context's install command to setup hooks in different harnesses.

## Decision

Add a `tenex-edge install` subcommand supporting three harness strategies (Claude Code JSON-merge 4 hooks + statusLine, Codex JSON-merge 3 hooks, opencode file-drop plugin). Dedup uses hook signature (`--host X --type Y`) rather than binary path prefix, so reinstalling after a path change replaces rather than accumulates entries. CLI mirrors `pc install`: --harness, --all, --dry-run, --status, --uninstall.

## Consequences

- Idempotent installs — signature-based dedup means binary path changes (e.g. cargo install → dev build) replace old hooks instead of accumulating duplicates
- statusLine entries are also deduped on reinstall
- Three harnesses supported out of the box without new code per harness (declarative strategy pattern)
- opencode integration uses include_str! bake of the plugin TS file, creating a compile-time coupling to integrations/opencode/tenex-edge.ts

## Open Tail

- No interactive multiselect picker yet (proactive-context has one); current flow uses --harness or --all flags only
- Uninstall path exists but was not smoke-tested in this session

## Evidence

- transcript lines 1-1
- transcript lines 310-314
- transcript lines 417-428
- transcript lines 485-494

