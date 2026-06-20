---
type: episode-card
date: 2026-06-15
session: 9c78b46a-3169-42eb-84c1-228a6c2f6589
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9c78b46a-3169-42eb-84c1-228a6c2f6589.jsonl
salience: root-cause
status: active
subjects:
  - tmux-spawn
  - project-resolution
  - tui-session-creation
supersedes: []
related_claims: []
source_lines:
  - 7-7
  - 154-154
  - 205-229
captured_at: 2026-06-18T00:34:30Z
---

# Episode: Tmux spawn uses selected project tab instead of process cwd

## Prior State

When spawning a new tmux session from the TUI on a spawnable agent, the project was resolved from `std::env::current_dir()` — the TUI process's working directory — regardless of which project tab was selected. The daemon's `spawn_agent` → `project_abs_path` would then correctly map that project name to its directory, but it was receiving the wrong project name to begin with.

## Trigger

User reported: creating a new session creates it within the PWD instead of within the selected project's directory.

## Decision

Changed the spawn handler to resolve the project from the active project tab filter (`pf`), falling back to cwd-resolution only on the unfiltered 'all projects' tab: `pf.map(str::to_string).unwrap_or_else(|| crate::project::resolve(&std::env::current_dir().unwrap_or_default()))`

## Consequences

- Spawning from a project-specific tab now correctly creates the session in that project's directory
- The 'all projects' tab still falls back to cwd, which may be semantically wrong but is at least no worse than before
- Fix landed inside a concurrent agent's commit (8da7494e) rather than its own dedicated commit due to a race on master

## Open Tail

- Should the 'all projects' tab disallow spawning or prompt for a project instead of falling back to cwd?

## Evidence

- transcript lines 7-7
- transcript lines 154-154
- transcript lines 205-229

