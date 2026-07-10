---
type: episode-card
date: 2026-07-10
session: 4d78f854-d5e5-4a11-b0d4-358f33111d15
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d78f854-d5e5-4a11-b0d4-358f33111d15.jsonl
salience: architecture
status: superseded
subjects:
  - workflow-storage
  - chief-of-staff
  - source-of-truth
supersedes: []
related_claims: []
source_lines:
  - 42-48
  - 263-267
  - 296-301
  - 305-321
  - 352-363
captured_at: 2026-07-10T07:34:09Z
---

# Episode: Workflows must live in a git repo, not a local home directory

## Prior State

Chief-of-staff workflow definitions lived as loose files in ~/.agents/homes/chief-of-staff/workflows/ — a plain local directory, not version-controlled, not shared across machines.

## Trigger

User directive: Pablo instructed that workflows should be moved into the touch-grass tracking repo and the home-dir path replaced with a symlink, so workflows carry over between computers. He also suggested the existing workflows.py script detect a non-symlink directory and warn.

## Decision

Workflows are now classified as durable operating knowledge that must live in a git repo. agent.yaml instructions were updated to codify that once a tracking repo exists, workflows/ should be moved into it and the home-dir path symlinked. scripts/workflows.py was modified to check on every invocation whether workflows/ is a plain directory vs a symlink and emit a loud stderr warning if it is not.

## Consequences

- agent.yaml now contains explicit doctrine: 'Workflows are durable operating knowledge, not local scratch state' with instructions to move and symlink without waiting to be asked.
- workflows.py enforces the doctrine at runtime — any agent session using a plain-directory workflows/ path will see a prominent warning on every script invocation.
- The actual migration of existing loose workflow files on this machine has not yet been performed; the warning will fire until the symlink move is done.
- PR #2 opened against touch-grass with these changes to agent.yaml, workflows.py, and docs/product/chief-of-staff.md (product-note entry per repo convention).

## Open Tail

- Loose workflows in ~/.agents/homes/chief-of-staff/workflows/ on this machine still need to be moved into the tracking repo (likely pablof7z/everything) and symlinked — deferred until PR merges and destination is confirmed.

## Evidence

- transcript lines 42-48
- transcript lines 263-267
- transcript lines 296-301
- transcript lines 305-321
- transcript lines 352-363

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-workflows-must-live-in-a-git.json`](transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-workflows-must-live-in-a-git.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-workflows-must-live-in-a-git.json`](transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-workflows-must-live-in-a-git.json)
