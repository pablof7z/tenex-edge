---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: product
status: active
subjects:
  - who-command
  - other-projects-display
supersedes:
  - 2026-06-09-2-who-scope-all-projects-current-project
related_claims: []
source_lines:
  - 211-220
  - 298-360
captured_at: 2026-06-12T20:14:05Z
---

# Episode: `who` command shows project summaries instead of per-agent listings for other projects

## Prior State

The `who` command's 'other projects' section listed each individual agent (e.g., `claude@tenex-edge`, `codex@tenex-edge`) making it verbose and duplicative.

## Trigger

User requested that the other-projects section show only project names and metadata, not individual agents: 'should only list the projects and their metadata if available'.

## Decision

Changed the other-projects section to render one line per project (name + description/summary) instead of one line per agent.

## Consequences

- Output is more concise: shows project identity rather than enumerating agents.
- The `OtherProjectSummary` struct already had an `about` field for project descriptions, now rendered.

## Open Tail

*(none)*

## Evidence

- transcript lines 211-220
- transcript lines 298-360

