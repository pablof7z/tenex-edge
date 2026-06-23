---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: root-cause
status: active
subjects:
  - project-slug-resolution
  - git-worktree
supersedes: []
related_claims: []
source_lines:
  - 1-69
  - 147-161
captured_at: 2026-06-12T20:14:05Z
---

# Episode: Git worktree project slug unification via --git-common-dir

## Prior State

Project slug resolution used `git rev-parse --show-toplevel` to determine the repo name, which returns the worktree-specific path for linked worktrees. This caused agents in a worktree to resolve a different project slug (e.g., `agent-a7dd3273a093df183`) than agents in the main repo (`TENEX-ff3ssq`).

## Trigger

User reported that agents in a git worktree and its main repo were scoped to different projects despite sharing the same repository.

## Decision

Changed `git_toplevel()` to use `git rev-parse --git-common-dir` (which returns the shared `.git` directory path, consistent across all worktrees) instead of `--show-toplevel`, then extract the parent directory's basename as the project slug.

## Consequences

- All worktrees of the same repository now resolve to the same project slug.
- Existing stale peer_sessions with worktree-derived project names in the database still show as separate projects until they age out or are pruned.
- The documented priority order (.tenex/project.json → git repo name → cwd basename) now works as intended for worktree scenarios.

## Open Tail

- Stale sessions in state.db under worktree-derived project names may need manual cleanup.

## Evidence

- transcript lines 1-69
- transcript lines 147-161

