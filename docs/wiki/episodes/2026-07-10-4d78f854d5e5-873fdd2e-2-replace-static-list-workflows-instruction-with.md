---
type: episode-card
date: 2026-07-10
session: 4d78f854-d5e5-4a11-b0d4-358f33111d15
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d78f854-d5e5-4a11-b0d4-358f33111d15.jsonl
salience: architecture
status: active
subjects:
  - session-start-script
  - session-start-entrypoint
  - setup-gate
  - self-evolution
supersedes: []
related_claims: []
source_lines:
  - 528-531
  - 566-577
  - 607-638
  - 693-714
captured_at: 2026-07-10T08:00:31Z
---

# Episode: Replace static 'list workflows' instruction with scripted session-start entrypoint

## Prior State

agent.yaml instructed the agent to list available workflows at the start of each session and choose the closest one — a static instruction permanently in context.

## Trigger

User idea (line 528): instead of telling the agent to list workflows, run a script that programmatically gives tailored context — inject SETUP.md if home isn't yet tracked in a repo, or inject the session brief (tracked location, workflow list, standing notes) if already set up. This gives flexibility to guide self-evolution, control cronjobs, and track things proactively.

## Decision

Created scripts/session_start.py as the single session-start entrypoint. agent.yaml now says 'run scripts/session_start.py' instead of 'list workflows.' The script: (1) ensures home dir exists, (2) if home is not a symlink → injects references/SETUP.md (a runbook guiding the agent through repo bootstrap, migration, and symlinking) plus a live listing of home dir contents, (3) if home is symlinked → injects a session brief (tracked git location, workflow list via workflows.py, and BRIEF.md standing notes).

## Consequences

- Self-healing onboarding: fresh machines auto-guide through repo setup with no human memory required
- BRIEF.md in home dir becomes a standing note that resurfaces every session — where proactive state like 'inbox-monitor loop alive' or 'decisions waiting on Pablo' lives
- Future proactive features (daily-report pointer, heartbeat checks, stale-blocker scan) get added in build_brief() in the script, keeping agent.yaml lean
- Growth happens in the script, not in the prompt — consistent with the constraint of not bloating agent.yaml context
- SETUP.md and session_start.py added to agent.yaml resources section

## Open Tail

- Tracking repo destination (pablof7z/everything?) not yet confirmed — migration not yet run
- BRIEF.md format and proactive sections are future work to be added in build_brief()

## Evidence

- transcript lines 528-531
- transcript lines 566-577
- transcript lines 607-638
- transcript lines 693-714

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d78f854d5e5-873fdd2e-2-replace-static-list-workflows-instruction-with.json`](transcripts/2026-07-10-4d78f854d5e5-873fdd2e-2-replace-static-list-workflows-instruction-with.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-2-replace-static-list-workflows-instruction-with.json`](transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-2-replace-static-list-workflows-instruction-with.json)
