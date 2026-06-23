---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: product
status: active
subjects:
  - who-renderer
  - agent-context-format
  - tty-detection
supersedes:
  - 2026-06-16-1-who-command-splits-into-dual-renderers
related_claims: []
source_lines:
  - 155-180
  - 275-291
  - 355-401
captured_at: 2026-06-16T10:37:51Z
---

# Episode: Split who renderer into human vs agent output formats

## Prior State

The `who` command had a single renderer for both human terminal output and agent turn-start fabric injection. `render_who_plain` was just an ANSI-stripping shim over the same layout — no structural difference between the two audiences.

## Trigger

User provided a restructured mockup with sections, tables, and embedded command instructions, then explicitly directed: "we need to separate in what we will render for the agent (no color, markup that works better for an agent) and human."

## Decision

Two distinct renderers off the same `WhoSnapshot`: `render_who_once` (human — colorized, compact, column-aligned) and `render_who_agent` (agent — markdown headings, pipe-delimited table, embedded `inbox send`/`inbox new-session` commands, em-dash for empty titles, no ANSI). Selection is automatic via TTY detection: terminal → human; piped/captured → agent. Turn-start fabric injection always uses the agent format. Old `render_who_plain`, `render_who_row`, and `status_colored` removed.

## Consequences

- Agent-facing `who` output is now parseable structured markdown with actionable command examples, not stripped ANSI
- Turn-start fabric block (injected into every agent's context) now carries discoverable `inbox send` and `inbox new-session` instructions
- `[spawnable via …]` suffix removed from agent list — `developer` = `claude --dangerously-skip-permissions` no longer visible
- Section order changed: Sessions → Agents → Other projects (was Sessions → Other projects → Agents)
- Both renderers share `row_cells()` helper so they cannot disagree on data

## Open Tail

- Other projects section now shows names only; project `about`/description dropped. Restorable on request.

## Evidence

- transcript lines 155-180
- transcript lines 275-291
- transcript lines 355-401

