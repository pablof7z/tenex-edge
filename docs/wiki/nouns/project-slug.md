---
type: noun-entry
slug: project-slug
name: "project (slug)"
origin: extracted
source_refs:
  - transcript:966-981
---

# project (slug)

Identified by a short slug, resolved from a working directory by: (1) Git repo name (via `git rev-parse --git-common-dir`, so a repo and all worktrees resolve to the same slug), (2) `~/.mosaico/projects.json` JSON map of slugs to absolute paths (nearest ancestor wins), or (3) `Err(NoProject)`.
