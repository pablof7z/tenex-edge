---
type: episode-card
date: 2026-06-16
session: a88513d3-754f-4369-b440-72c8d29331e2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a88513d3-754f-4369-b440-72c8d29331e2.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge-who-renderer
  - agent-context-format
  - tty-detection
supersedes:
  - 2026-06-16-1-who-command-output-splits-into-agent
related_claims: []
source_lines:
  - 1-54
  - 155-178
  - 275-293
  - 351-407
captured_at: 2026-06-16T10:31:52Z
---

# Episode: who command splits into dual renderers: human vs agent

## Prior State

The `who` command had a single renderer for both humans and agents. The agent-facing output (`render_who_plain`) was just ANSI-stripped text from the same layout — no structural markdown, no actionable commands embedded, no distinction between consumer needs.

## Trigger

User requested a restructured output with labeled sections (# Sessions, # Agents, # Other projects), markdown tables, and embedded command hints. When asked about scope, user explicitly directed: 'we need to separate in what we will render for the agent (no color, markup that works better for an agent) and human' — rejecting the old strip-ANSI approach.

## Decision

Split into two distinct renderers sharing a `row_cells()` helper: (1) `render_who_once` — human, colorized, column-aligned; (2) `render_who_agent` — markdown headings + table + embedded `tenex-edge inbox send` / `tenex-edge tmux spawn` commands, no ANSI. TTY auto-detection selects between them: terminal → human, piped/captured → agent. Turn-start fabric injection now uses `render_who_agent` with a lead-in line. Old `render_who_plain` (ANSI shim), `render_who_row`, and `status_colored` are removed.

## Consequences

- Agent turn-start context now contains structured markdown with actionable command instructions, changing what agents 'see' at every turn start
- The `[spawnable via …]` suffix is dropped entirely — `developer@laptop` being `claude --dangerously-skip-permissions` is no longer visible in any output surface
- The actual spawn command surfaced is `tenex-edge tmux spawn --agent <slug>`, not the user's sketched `inbox new-session` (which doesn't exist)
- Empty session titles render as em-dash in agent output, not blank
- Tests rewritten for dual-renderer layout; blocked from running by a concurrent session's incomplete DomainEvent::ChatMessage wiring in server.rs

## Open Tail

- Whether to add an `inbox new-session` alias as the user originally sketched, versus the existing `tmux spawn` command
- Whether to restore project descriptions in the Other projects section (currently dropped to names-only)
- Tests cannot be verified until the concurrent session finishes wiring ChatMessage match arms

## Evidence

- transcript lines 1-54
- transcript lines 155-178
- transcript lines 275-293
- transcript lines 351-407

