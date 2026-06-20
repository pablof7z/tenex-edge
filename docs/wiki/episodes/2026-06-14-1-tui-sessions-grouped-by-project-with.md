---
type: episode-card
date: 2026-06-14
session: 656e1e6b-2569-42da-8844-768a5e74788e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/656e1e6b-2569-42da-8844-768a5e74788e.jsonl
salience: product
status: active
subjects:
  - tenex-edge-tmux-tui
  - project-tabs
  - session-navigation
supersedes: []
related_claims: []
source_lines:
  - 1-6
  - 379-384
  - 515-519
captured_at: 2026-06-18T00:27:48Z
---

# Episode: TUI sessions grouped by project with prioritized tabs and fuzzy search

## Prior State

Sessions across all projects shown in a single flat list with no project separation; impossible to tell which project a session belongs to at a glance

## Trigger

User requested project-based tabs so sessions could be distinguished by project, then followed up that too many projects appeared, requesting live-first ordering, 7-day hiding, and fuzzy search (/) to find dormant projects

## Decision

Sessions are now organized into per-project tabs (←/→ to switch). Tab bar shows [All] plus per-project tabs. Projects with live sessions appear first (alphabetically), then recently-active projects (within 7 days). Projects dormant >7 days are hidden from the tab bar but reachable via '/' fuzzy search overlay (↑/↓ move, Enter jumps, Esc cancels). Selecting a hidden project temporarily injects it into visible tabs until next refresh unless activity resumes.

## Consequences

- In 'All' tab, sessions are labeled slug@project so project is always visible
- Agents (spawnable rows) appear in all tabs since they're cross-project
- Tab ordering recomputes on every 2s refresh
- Hidden projects are dimmed in search results; visible ones listed first

## Open Tail

*(none)*

## Evidence

- transcript lines 1-6
- transcript lines 379-384
- transcript lines 515-519

