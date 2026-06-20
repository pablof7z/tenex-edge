---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: root-cause
status: active
subjects:
  - project-slug
  - git-worktree
supersedes: []
related_claims: []
source_lines:
  - 1-69
  - 147-197
captured_at: 2026-06-17T23:58:37Z
---

# Episode: Worktree project slug resolves to wrong project via git --show-toplevel

## Prior State

Project slug resolution used `git rev-parse --show-toplevel` to derive the repo name, which returns each worktree's own path (e.g., `agent-a7dd3273a093df183`) rather than the shared repo name, causing agents in worktrees to be scoped to a different project than the main repo.

## Trigger

User reported that agents in a git worktree and the main repo reported as two different projects despite being the same repository.

## Decision

Switch `git_toplevel()` to use `git rev-parse --git-common-dir` instead of `--show-toplevel`. The common-dir is shared across all worktrees, so its basename correctly resolves to the same project slug regardless of which worktree the agent runs in.

## Consequences

- Main repo and all its worktrees now resolve to the same project slug (e.g., TENEX-ff3ssq)
- Existing daemon must be restarted to pick up the new binary — stale processes continue using old resolution
- The documentation already stated git repo name should be shared across worktrees; this fix aligns implementation with the stated design

## Open Tail

*(none)*

## Evidence

- transcript lines 1-69
- transcript lines 147-197

