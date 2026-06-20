---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: product
status: active
subjects:
  - who-command
  - peer-display
supersedes: []
related_claims: []
source_lines:
  - 211-367
captured_at: 2026-06-17T23:58:37Z
---

# Episode: who command shows project summaries instead of individual agents in other-projects

## Prior State

The `who` command's 'other agents in other projects' section listed every individual agent (e.g., `claude@tenex-edge`, `codex@tenex-edge`), producing redundant lines when multiple agents share a project.

## Trigger

User directed that the other-projects section should list only the project names and their metadata, not each agent.

## Decision

Changed the other-projects rendering to group by project and show one line per project (name + description metadata), removing per-agent detail.

## Consequences

- Output is more compact and less noisy when multiple agents run in the same project
- Individual agent details in other projects are no longer visible from this view

## Open Tail

*(none)*

## Evidence

- transcript lines 211-367

