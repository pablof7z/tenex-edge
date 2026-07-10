---
type: episode-card
date: 2026-07-10
session: a358c40d-687f-4e0f-b383-aca7cfebb243
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a358c40d-687f-4e0f-b383-aca7cfebb243.jsonl
salience: root-cause
status: active
subjects:
  - proactive-context
  - git-hook-architecture
  - wiki-auto-commit
supersedes: []
related_claims: []
source_lines:
  - 139-214
captured_at: 2026-07-10T15:20:44Z
---

# Episode: proactive-context auto-commit hook is structurally blind to capture-time regenerations

## Prior State

Belief that proactive-context's managed git hook would automatically commit regenerated wiki artifacts whenever the capture process rewrote docs/wiki/

## Trigger

User observed that wiki artifacts were left uncommitted after a capture regeneration, blocking git pull with pull.rebase=true — asked why the auto-commit hook didn't fire

## Decision

Root-cause diagnosis: the managed hook is post-commit only, meaning it fires as a side-effect of an explicit commit, not when the capture process writes files. Fast-forwards and capture-time file writes produce no commit event, so the hook never runs in those windows. The hook is structurally blind to the gap between regeneration and the next manual commit.

## Consequences

- Any wiki regeneration that occurs without an intervening manual commit leaves docs/wiki/ dirty and uncommitted
- With pull.rebase=true, a pull that triggers regeneration cannot auto-commit because no post-commit fires during fast-forward or rebase
- When the manual commit finally does fire, the hook no-ops because docs/wiki is already clean (already committed)
- Two structural gaps identified: (1) no post-merge/post-rewrite hook to cover pull-time regenerations, (2) auto-commit is not attached to the capture step itself

## Open Tail

- Proposed but not yet decided: have the capture routine self-commit immediately after writing, or add post-merge/post-rewrite hooks — user has not responded to the proposal

## Evidence

- transcript lines 139-214

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-a358c40d687f-638bf354-1-proactive-context-auto-commit-hook-is.json`](transcripts/2026-07-10-a358c40d687f-638bf354-1-proactive-context-auto-commit-hook-is.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-a358c40d687f-638bf354-1-proactive-context-auto-commit-hook-is.json`](transcripts/raw/2026-07-10-a358c40d687f-638bf354-1-proactive-context-auto-commit-hook-is.json)
