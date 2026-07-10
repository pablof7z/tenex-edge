---
type: episode-card
date: 2026-07-10
session: 4d78f854-d5e5-4a11-b0d4-358f33111d15
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d78f854-d5e5-4a11-b0d4-358f33111d15.jsonl
salience: architecture
status: active
subjects:
  - chief-of-staff-home-tracking
  - symlink-doctrine
  - workflows.py-symlink-check
supersedes:
  - 2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-dir-must
related_claims: []
source_lines:
  - 42-48
  - 366-371
  - 436-467
  - 494-526
captured_at: 2026-07-10T08:00:31Z
---

# Episode: Chief-of-staff home directory must be symlinked into a git repo, not a local dir

## Prior State

Agent workflows and operating state lived as loose files in ~/.agents/homes/chief-of-staff/ on the local machine, not version-controlled, not portable across machines.

## Trigger

User directive (lines 42–48): workflows should move into a git repo and ~/.agents/homes/chief-of-staff/workflows should become a symlink so state is shared across computers. User correction (lines 366–371): mirror the same path structure in the repo (<repo>/.agents/homes/chief-of-staff/), remove the instruction from agent.yaml (don't carry it forever in context), and link the whole chief-of-staff home — not just the workflows subdirectory.

## Decision

The entire ~/.agents/homes/chief-of-staff/ directory must be symlinked into a tracking repo at <repo>/.agents/homes/chief-of-staff/, not just workflows/. The nudge/warning lives only in scripts/workflows.py (checks on every run, prints stderr warning when home is a plain dir, silent when symlinked) — no persistent instruction added to agent.yaml's context. The initial approach of putting prose in agent.yaml and scoping the check to workflows/ only was reverted.

## Consequences

- agent.yaml instructions block stays lean — no permanent nudge text in agent context
- Warning is ephemeral: only surfaces when workflows.py runs and disappears once home is symlinked
- Whole home dir (workflows, references, state) becomes version-controlled, not just workflows
- State directory contents (logs, cursors) will need .gitignore when migrated
- Path structure mirrors local layout inside the tracking repo

## Open Tail

- Actual migration of the 6 live workflows from ~/.agents/homes/chief-of-staff/ into the tracking repo (pablof7z/everything?) not yet performed — awaiting user confirmation of destination repo

## Evidence

- transcript lines 42-48
- transcript lines 366-371
- transcript lines 436-467
- transcript lines 494-526

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-directory-must.json`](transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-directory-must.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-directory-must.json`](transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-directory-must.json)
