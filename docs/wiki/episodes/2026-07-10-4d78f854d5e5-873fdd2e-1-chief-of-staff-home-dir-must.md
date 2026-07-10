---
type: episode-card
date: 2026-07-10
session: 4d78f854-d5e5-4a11-b0d4-358f33111d15
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d78f854-d5e5-4a11-b0d4-358f33111d15.jsonl
salience: architecture
status: active
subjects:
  - chief-of-staff-home-dir
  - workflows-version-control
  - symlink-nudge
supersedes:
  - 2026-07-10-4d78f854d5e5-873fdd2e-1-workflows-must-live-in-a-git
related_claims: []
source_lines:
  - 42-48
  - 309-321
  - 366-371
  - 436-461
  - 514-524
captured_at: 2026-07-10T07:47:32Z
---

# Episode: Chief-of-staff home dir must be symlinked into a git repo, not loose local files

## Prior State

The chief-of-staff agent's home directory (~/.agents/homes/chief-of-staff/) — containing workflows, references, state, and scripts — existed as loose local files on each machine. No version control, no cross-machine sharing. No symlink detection or nudge existed.

## Trigger

User directive: workflows (and by extension the whole agent home) are durable operating knowledge that must live in a git repo so they carry over between machines. User further corrected the initial implementation: (1) mirror the local path structure in the repo as <repo>/.agents/homes/chief-of-staff/ rather than an ad-hoc path, (2) keep the nudge out of agent.yaml instructions so it doesn't permanently pollute agent context, (3) link the entire chief-of-staff home dir, not just the workflows subdirectory.

## Decision

The entire ~/.agents/homes/chief-of-staff/ directory should be version-controlled by moving its contents into a tracking repo at <tracking-repo>/.agents/homes/chief-of-staff/ and replacing the local path with a symlink. The symlink-detection warning lives exclusively in scripts/workflows.py (runtime, ephemeral), never in agent.yaml's permanent instructions. The check fires on every workflows.py invocation and warns on stderr when the home dir is a plain directory; it goes silent once symlinked.

## Consequences

- agent.yaml instructions were reverted to original — no permanent context bloat from the nudge
- workflows.py now checks the whole home dir (~/.agents/homes/chief-of-staff/), not just the workflows/ subdirectory
- Warning message directs users to mirror the local path structure: <tracking-repo>/.agents/homes/chief-of-staff/
- Product-notes doc (docs/product/chief-of-staff.md) retains both the original decision and the same-day correction, preserving history per repo convention
- The actual migration of existing loose files on this machine is not yet performed — pending confirmation of the tracking repo destination
- An intermediate approach (putting prose in agent.yaml, checking only workflows/) was implemented, reverted, and replaced — now historical

## Open Tail

- Existing loose files in ~/.agents/homes/chief-of-staff/ on this machine still need to be moved into the tracking repo and symlinked once the destination (likely pablof7z/everything) is confirmed
- Stray vim swapfile agents/chief-of-staff/.agent.yaml.swp noted in the repo working tree, left uncleaned

## Evidence

- transcript lines 42-48
- transcript lines 309-321
- transcript lines 366-371
- transcript lines 436-461
- transcript lines 514-524

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-dir-must.json`](transcripts/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-dir-must.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-dir-must.json`](transcripts/raw/2026-07-10-4d78f854d5e5-873fdd2e-1-chief-of-staff-home-dir-must.json)
