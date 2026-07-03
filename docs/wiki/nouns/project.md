---
type: noun-entry
slug: project
name: "project"
origin: extracted
source_refs:
  - transcript:966-981
---

# project

Identified by a short slug resolved from a working directory via: (1) git repo name (shared across worktrees), (2) ~/.tenex-edge/projects.json slug→path map (the only way to register a non-git directory), or (3) Err(NoProject). No .tenex/project.json file exists; the map at ~/.tenex-edge/projects.json is the single source of truth for non-git projects.
