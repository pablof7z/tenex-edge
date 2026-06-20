---
title: Git Branch Merge Strategy
slug: git-branch-merge-strategy
topic: git-workflow
summary: Diverged branches must be resolved via proper merge, not force-push, to preserve all existing work on both sides
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-13
updated: 2026-06-17
verified: 2026-06-13
compiled-from: conversation
sources:
  - session:c55d561a-ccf5-4160-ab1d-d5946e9e400f
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
  - session:9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf
  - session:rollout-2026-06-14T13-19-49-019ec5a5-1119-76f0-a7e3-36bc985a31bd
  - session:rollout-2026-06-17T10-15-05-019ed46f-0289-7cf3-ae87-5a65210ee266
---

# Git Branch Merge Strategy

## Merge Strategy

Diverged branches must be resolved via proper merge, not force-push, to preserve all existing work on both sides. Branch synchronization with origin must not lose any work from either side. When two branches have significantly diverged with many commits, git merge is preferred over rebase to resolve conflicts in a single merge commit rather than replaying commit-by-commit. <!-- [^c55d5-1] -->


Commits scope only files directly modified for the current change, excluding unrelated pre-existing staged or dirty files. <!-- [^rollo-90] -->
## Conflict Resolution

A new git worktree is used to isolate and resolve merge conflicts without disturbing the main working directory. Rebase conflicts involving structural file changes (e.g., a file split into subdirectory modules) should be resolved by keeping the modular version and skipping the old monolithic code blocks when the changes are already captured in the new structure. Leftover unstaged artifacts from a merge (such as wiki/doc changes) must be committed before performing subsequent merges. <!-- [^c55d5-2] -->

## Cleanup

The local main branch dirty state was snapshotted to backup/local-main-wip-2026-06-13 (471 files pushed to origin) before fast-forwarding to origin/main. Already-merged worktrees fix/android-unit-test-compile (PR #430) and fix/nip55-identity-propagation (PR #422) were removed along with their local branches after confirming all work was present in origin/main. The popup branch (worktree-agent-a7aeac15bf94749ea) has been dropped and its worktree cleaned up. Stale extra worktrees and local branches with no unique commits relative to master are removed during queue cleanup. The nested worktree-agent branch is not resurrected wholesale because it has no unique committed work versus master and is scratch/stale; master is the source of truth.

<!-- citations: [^74fce-10] [^9bab9-1] [^rollo-47] -->
