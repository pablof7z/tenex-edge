---
type: episode-card
date: 2026-06-09
session: f9bdcf4c-c972-46ff-91b8-9e30785d3331
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f9bdcf4c-c972-46ff-91b8-9e30785d3331.jsonl
salience: reversal
status: active
subjects:
  - hook-dispatch
  - python-wrapper
  - tenex-edge-hook-subcommand
supersedes:
  - 2026-06-09-2-context-injection-moves-from-python-scripts
related_claims: []
source_lines:
  - 1-311
captured_at: 2026-06-17T23:50:24Z
---

# Episode: Abandon Python wrapper for direct binary hook invocation

## Prior State

All agent harness hooks dispatched through a Python wrapper script (te-hook.py) that read Codex/Claude Code hook JSON from stdin and delegated to tenex-edge. Codex config.template.toml used python3 __HOOK__ commands; Claude Code channel server comments and README referenced te-hook.py; wiki docs described the Python path.

## Trigger

User directive: 'configure codex/opencode/claudecode to use the new shape (we abandoned the python wrapper)'

## Decision

Eliminate the Python wrapper intermediary. All harness hooks now invoke tenex-edge hook --host <name> --type <hook-type> directly. Updated Codex config.template.toml, Codex README, Claude Code channel server.ts comments, Claude Code channel README, wiki docs, and live Claude Code settings.json to remove all Python references and use direct binary calls.

## Consequences

- No Python dependency required for hook dispatch; the Rust binary is the sole invocation point
- Codex __HOOK__ path-substitution step eliminated from install instructions
- Claude Code settings.json gained four tenex-edge hooks (SessionStart, UserPromptSubmit, Stop, SessionEnd) running alongside existing pc hooks
- te-hook.py file still exists on disk but is no longer referenced by any configuration or documentation

## Open Tail

- te-hook.py may need to be deleted or archived once all users have migrated

## Evidence

- transcript lines 1-311

