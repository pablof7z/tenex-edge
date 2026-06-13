---
title: Branch Management
slug: tenex-branch-management
topic: version-control
summary: Divergent branches must be resolved through a proper merge that preserves all work from both sides, not via a force-push.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-13
updated: 2026-06-13
verified: 2026-06-13
compiled-from: conversation
sources:
  - session:c55d561a-ccf5-4160-ab1d-d5946e9e400f
---

# Branch Management

## Branch Resolution

Divergent branches must be resolved through a proper merge that preserves all work from both sides, not via a force-push. <!-- [^c55d5-1] -->

Conflict resolution for significantly diverged branches must use a new git worktree rather than working directly on the main working tree. <!-- [^c55d5-2] -->

When two branches have significantly diverged, resolving conflicts via `git merge` in a single merge commit is preferred over a commit-by-commit rebase. <!-- [^c55d5-3] -->

When a rebased commit's changes are already captured by structural refactoring on the other branch (e.g., code moved into subdirectory modules), the HEAD (modular) version of the file is kept and the old code block is skipped. <!-- [^c55d5-4] -->
