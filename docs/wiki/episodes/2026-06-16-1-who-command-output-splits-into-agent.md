---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: architecture
status: superseded
subjects:
  - tenex-edge-who-render
  - agent-vs-human-output
supersedes: []
related_claims: []
source_lines:
  - 1-53
  - 117-153
  - 155-156
captured_at: 2026-06-16T10:19:39Z
---

# Episode: Who-command output splits into agent vs. human render paths

## Prior State

The `who` command used a single render path (render_who_once/render_who_plain) that mixed ANSI colors, ad-hoc section ordering, inline spawnable-via tags, and flat session lines — one output for both human CLI and agent turn-start context injection.

## Trigger

User requested a full reformat of `who` output (labeled project header, markdown table for sessions, reordered sections, dropped spawnable-via tags). Assistant flagged that the same renderer feeds both the interactive CLI and the turn-start fabric injection agents receive. User then directed: 'we need to separate in what we will render for the agent (no color, markup that works better for an agent) and human.'

## Decision

The `who` renderer will bifurcate into two distinct render paths: (1) a human-facing output with ANSI colors, section headers, and markdown-table layout; (2) an agent-facing output with no color and markup optimized for agent context consumption. The single-renderer approach is retired.

## Consequences

- Spawnable-via tags (e.g. `claude --dangerously-skip-permissions`) are removed from all visible output; the command/harness distinction is no longer surfaced to users or agents in `who`.
- Sessions section will use a markdown table (Agent | Session Id | Host | Session Title | Status) at minimum in the human path; the agent path may use a different structure.
- Section order becomes: Sessions → Agents (for new sessions) → Other projects, replacing the prior Sessions → Other projects → Agents order.
- The existing `render_who_plain` function (used for agent turn-start injection) must be replaced or diverged from rather than sharing the human render logic.
- Session title and idle/working status must be split into separate fields rather than the current combined `status_colored` string.

## Open Tail

- Exact markup format for the agent render path is not yet specified — plain text? JSON? Structured markdown without tables?
- Whether the agent path also gets the same section reordering and label changes, or a different information hierarchy
- Whether spawnable-via info should be available via a different command/flag even though it's removed from `who`

## Evidence

- transcript lines 1-53
- transcript lines 117-153
- transcript lines 155-156

