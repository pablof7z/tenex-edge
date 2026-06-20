---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: product
status: active
subjects:
  - who-renderer
  - agent-context-block
  - turn-start-injection
supersedes: []
related_claims: []
source_lines:
  - 155-180
  - 351-407
captured_at: 2026-06-18T00:40:35Z
---

# Episode: who output split into dual renderers (human vs agent)

## Prior State

Single renderer: `render_who_once` produced one output (colorized, flat lines), and `render_who_plain` was just an ANSI-stripped shim over the same layout. Both humans and agents received essentially the same information in the same format.

## Trigger

User's initial markdown sketch of structured output (headings, tables, embedded commands) was clearly agent-facing, followed by explicit directive: 'we need to separate in what we will render for the agent (no color, markup that works better for an agent) and human.' Design conversation then selected TTY auto-detection as the switch.

## Decision

Two genuinely different renderers sharing one `row_cells()` helper: `render_who_once` (human: bold headers, color, column-aligned sessions table, agents on one line, dotted other-projects) and `render_who_agent` (agent: no ANSI, markdown headings + `| Agent | Session | Host | Title | Status |` table, bulleted agent list, embedded `inbox send` / `new-session` command instructions). Auto-selected by TTY — terminal → human; piped/captured → agent. Turn-start fabric injection (`push_turn_fabric_block`) now uses the agent format.

## Consequences

- Agent context blocks are now structured markdown with actionable CLI commands embedded inline, parseable by LLMs without ANSI noise
- Old `render_who_plain` (ANSI strip shim), `render_who_row`, and `status_colored` all removed
- `row_cells()` shared helper ensures both renderers can't disagree on facts
- Section reordering: Sessions → Agents → Other projects (was Sessions → Other projects → Agents)
- `[spawnable via …]` tag dropped from agent display, losing the harness distinction (e.g. developer = claude --dangerously-skip-permissions)
- Empty session titles render as em-dash (—) in agent format

## Open Tail

- Whether to restore `[spawnable via …]` info somewhere (currently invisible)
- Whether to restore project `about`/description in Other projects (now names only)

## Evidence

- transcript lines 155-180
- transcript lines 351-407

