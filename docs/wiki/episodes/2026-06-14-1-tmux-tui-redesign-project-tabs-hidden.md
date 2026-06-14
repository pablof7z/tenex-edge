---
type: episode-card
date: 2026-06-14
session: 656e1e6b-2569-42da-8844-768a5e74788e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/656e1e6b-2569-42da-8844-768a5e74788e.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge
  - tmux-tui
  - session-display
supersedes: []
related_claims: []
source_lines:
  - 1-8
  - 315-321
  - 369-377
captured_at: 2026-06-14T16:05:15Z
---

# Episode: Tmux TUI redesign: project tabs, hidden exited sessions, label renames

## Prior State

Tmux TUI showed all sessions in a flat list regardless of project, always displayed exited sessions in a 'Resumable' section, and labeled unattached agents as 'Spawnable (no session)' with '[spawnable via claude]'

## Trigger

User directives: (1) sessions should be separated by project in tabs so the user can identify which project a session belongs to, (2) exited sessions should be hidden by default and require a keypress to reveal, (3) rename 'Spawnable (no session)' to 'Agents', (4) rename '[spawnable via claude]' to '[claude]'

## Decision

Redesign tmux TUI with: project-tabbed navigation (←/→ arrows) with [All] as default tab showing slug@project format; exited sessions hidden by default toggled with 'e' key; 'Spawnable (no session)' → 'Agents'; '[spawnable via claude]' → '[claude]'. Agents appear in all tabs since they are cross-project.

## Consequences

- Sessions are now grouped by project; users can scope view to a single project or see all with project annotations
- Exited/resumable sessions are opt-in visibility, reducing noise in the default view
- Label terminology simplified for clarity and brevity
- ResumeRow struct's 'alive' field removed as dead code after the exited-sessions visibility logic changed

## Open Tail

*(none)*

## Evidence

- transcript lines 1-8
- transcript lines 315-321
- transcript lines 369-377

