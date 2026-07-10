---
title: Git Merge Worktree Cleanup
slug: git-merge-worktree-cleanup
topic: repo-discipline
summary: After a merge succeeds, attempting to delete the local branch may fail if a git worktree still references that branch
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-10
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:4e6163df-c3cd-4d85-99ad-041cd0ca9701
  - session:af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
---

# Git Merge Worktree Cleanup

## Worktree Blocks Branch Deletion After Merge

After a merge succeeds, attempting to delete the local branch may fail if a git worktree still references that branch. The worktree's checkout of the branch prevents git from removing it, so the worktree must be removed (or switched to a different branch) before the branch can be deleted. <!-- [^4e616-7612e] -->

## Merge Strategy

The repo uses standard merge commits, not squash merges, for PR merges. <!-- [^af454-f066b] -->
