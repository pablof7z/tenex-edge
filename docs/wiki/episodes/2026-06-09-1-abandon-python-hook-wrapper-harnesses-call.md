---
type: episode-card
date: 2026-06-09
session: f9bdcf4c-c972-46ff-91b8-9e30785d3331
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f9bdcf4c-c972-46ff-91b8-9e30785d3331.jsonl
salience: architecture
status: active
subjects:
  - hook-dispatch
  - codex-integration
  - claude-code-integration
  - opencode-integration
supersedes: []
related_claims: []
source_lines:
  - 1-3
  - 23-59
  - 122-159
  - 163-189
  - 237-264
  - 266-298
  - 303-311
  - 313-451
captured_at: 2026-06-12T20:04:49Z
---

# Episode: Abandon Python hook wrapper — harnesses call tenex-edge binary directly

## Prior State

All three agent harness integrations (Codex, Claude Code, OpenCode) dispatched hooks through a Python wrapper (te-hook.py) that read JSON from stdin and delegated to `tenex-edge hook --host <name> --type <hook-type>`. Codex's config.template.toml used `python3 "__HOOK__" <hook-type>` with a path-substitution step. Claude Code's channel server.ts and README still referenced te-hook.py in comments and docs.

## Trigger

User directive: 'configure codex/opencode/claudecode to use the new shape (we abandoned the python wrapper)' — the Python wrapper had already been replaced by a direct Rust binary invocation pattern for some harnesses, but Codex config and various docs/comments still referenced the old shape.

## Decision

All harness integrations now call the `tenex-edge` Rust binary directly via `tenex-edge hook --host <name> --type <hook-type>`. The Python wrapper (te-hook.py) is no longer the dispatch mechanism for any harness. Codex config.template.toml, Codex README, Claude Code settings.json, channel server.ts comments, channel README, and the wiki doc were all updated to reflect direct binary invocation and remove stale te-hook.py references.

## Consequences

- Codex config no longer requires the __HOOK__ path-substitution install step
- Codex README rewritten to document direct binary invocation instead of Python wrapper dispatch
- Claude Code user settings.json now includes tenex-edge hooks alongside existing pc hooks (both coexist)
- Channel server.ts comments and channel README cleaned of stale te-hook.py references
- Wiki host-adapter doc updated to remove stale te-hook.py path claim
- OpenCode integration was already using the new shape — no change needed

## Open Tail

- te-hook.py file still exists on disk but is now orphaned — may warrant deletion

## Evidence

- transcript lines 1-3
- transcript lines 23-59
- transcript lines 122-159
- transcript lines 163-189
- transcript lines 237-264
- transcript lines 266-298
- transcript lines 303-311
- transcript lines 313-451

