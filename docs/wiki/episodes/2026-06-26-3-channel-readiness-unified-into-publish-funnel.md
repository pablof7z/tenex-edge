---
type: episode-card
date: 2026-06-26
session: b429fe81-7956-4a43-a87f-94e1799bf6e3
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b429fe81-7956-4a43-a87f-94e1799bf6e3.jsonl
salience: architecture
status: active
subjects:
  - channel-readiness
  - nip29-lifecycle
  - publish-funnel
  - launch-channel-bug
supersedes: []
related_claims: []
source_lines:
  - 186-190
  - 586-886
captured_at: 2026-06-26T07:57:03Z
---

# Episode: Channel readiness unified into publish-funnel gate

## Prior State

Channel-existence and membership checks were scattered across `open_project` and `ensure_session_room` with duplicated logic; `launch --channel` did not ensure parent group creation, leaving channels in an inconsistent state.

## Trigger

User observation: running `tenex-edge launch --channel wasm developer` created the subgroup but not the parent project channel. Opus architecture analysis identified that all domain publishes converge on three methods in `Nip29Provider` and proposed unified gating as the systemic fix.

## Decision

Create `ensure_channel_ready(ctx: &ChannelCtx)` in `src/fabric/nip29/readiness.rs` and insert at three publish-funnel points: `publish`, `publish_checked`, and `set_status` on `Nip29Provider`. Implement with in-memory TTL'd cache (fast path) → local read-model check → single-flighted relay fetch (slow path). Accept `parent_hint` in `ChannelCtx` to fix `launch --channel` by ensuring parent first. Collapse `open_project` and `ensure_session_room` into callers of the unified gate.

## Consequences

- Eliminates three copies of channel-lifecycle logic, reducing maintenance burden and inconsistency risk
- Every domain publish now gates on channel readiness without per-callsite opt-in
- Fixes `launch --channel` bug by ensuring parent group is created first when hint is provided
- Single source-of-truth for channel prerequisites; future channel-lifecycle changes only need one update location
- Fail-open design (gate never blocks publish; relay verdict is the real backstop) maintains resilience

## Open Tail

- Implementation pending (design complete); requires coding in `readiness.rs` and insertion at three `Nip29Provider` call sites

## Evidence

- transcript lines 186-190
- transcript lines 586-886

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-3-channel-readiness-unified-into-publish-funnel.json`](transcripts/2026-06-26-3-channel-readiness-unified-into-publish-funnel.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-3-channel-readiness-unified-into-publish-funnel.json`](transcripts/raw/2026-06-26-3-channel-readiness-unified-into-publish-funnel.json)
