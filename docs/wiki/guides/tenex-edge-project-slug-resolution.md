---
title: tenex-edge Project Slug Resolution
slug: tenex-edge-project-slug-resolution
topic: tenex-edge
summary: Project slug resolution uses `git rev-parse --git-common-dir` (instead of `--show-toplevel`) to extract the shared repository directory's basename, ensuring git
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:435ec383-d607-459b-a712-a00ed4decaa7
---

# tenex-edge Project Slug Resolution

## Project Slug Resolution

Project slug resolution uses `git rev-parse --git-common-dir` (instead of `--show-toplevel`) to extract the shared repository directory's basename, ensuring git worktrees and their main repo resolve to the same project slug. <!-- [^435ec-3] -->
