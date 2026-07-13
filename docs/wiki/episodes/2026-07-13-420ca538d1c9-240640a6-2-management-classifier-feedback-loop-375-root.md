---
type: episode-card
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
salience: root-cause
status: active
subjects:
  - management-classifier
  - kind9-loop
  - issue-375
  - codename-resolution
supersedes: []
related_claims: []
source_lines:
  - 16-20
  - 35-53
  - 824-839
captured_at: 2026-07-13T08:15:49Z
---

# Episode: Management classifier feedback loop (#375) root-caused and closed

## Prior State

The daemon's management classifier dispatched any kind:9 event p-tagging the management pubkey — including ordinary agent prose that failed strict parsing. A failed parse published an error event, the agent replied to the management identity, and the cycle repeated every ~20s. Daemon restarts replayed the backlog and amplified the loop. Additionally, agents were observed replying to themselves: the p-tag on their events pointed back at the sender, and @mention text used a `codex-<name>` prefixed form that didn't match the actual `-codex`-suffixed member handles, causing the codename resolver to misroute or self-resolve.

## Trigger

Channel chatter flagged the daemon stuck in a management-command dispatch loop. codex-indigo-lima-590 root-caused it as a pre-parse classifier fault: the classifier was too broad, dispatching non-command kind:9 events. User separately observed agents ignoring p-tags, and investigation confirmed the visible @mention and p-tag routing had diverged — two different codename-emission paths produce different handle formats, one resolves and one doesn't.

## Decision

#375 was closed by PR #399 (`fix(mgmt): gate management classifier on command-shape to close kind:9 loop`) on 2026-07-12. The classifier now gates on command-shape rather than dispatching any kind:9 event to the management pubkey.

## Consequences

- The ~20s management feedback loop is eliminated.
- A `daemon restart` subcommand (#398) was added, and #391 fixed orphan live supervisors + false-revive ghosts (the C4 zombie-online symptom observed during diagnosis).
- The codename format divergence (`codex-<name>` prefix vs `-codex` suffix) was identified as sharing the same substrate as #375 — kind:9 events landing on wrong p-tags — but no separate fix was filed for the handle-format bug.
- The specific slate-falcon incident that triggered the investigation is stale — that session is long gone across restarts.

## Open Tail

- The codename-emission path divergence (two paths producing different handle formats) was diagnosed but not tracked as a separate issue; it may be fully resolved by the #399 classifier gate, or may still produce misrouted @mentions in non-management contexts.

## Evidence

- transcript lines 16-20
- transcript lines 35-53
- transcript lines 824-839

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-13-420ca538d1c9-240640a6-2-management-classifier-feedback-loop-375-root.json`](transcripts/2026-07-13-420ca538d1c9-240640a6-2-management-classifier-feedback-loop-375-root.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-13-420ca538d1c9-240640a6-2-management-classifier-feedback-loop-375-root.json`](transcripts/raw/2026-07-13-420ca538d1c9-240640a6-2-management-classifier-feedback-loop-375-root.json)
