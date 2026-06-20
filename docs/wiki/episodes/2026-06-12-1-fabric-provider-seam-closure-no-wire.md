---
type: episode-card
date: 2026-06-12
session: 0bc06206-1f30-4e35-8373-f31d0f5c1dcc
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/0bc06206-1f30-4e35-8373-f31d0f5c1dcc.jsonl
salience: architecture
status: active
subjects:
  - fabric-provider-seam
  - wire-publish-surface
supersedes:
  - 2026-06-09-1-replace-codec-seam-with-fabric-provider
related_claims: []
source_lines:
  - 120-170
  - 4400-4660
  - 4715-4722
  - 5272-5278
captured_at: 2026-06-18T00:13:21Z
---

# Episode: Fabric provider seam closure: no wire shape above the provider

## Prior State

Session engine, RPC handlers, TurnReply, user prompts, doctor probe, and project edit all accessed codec+transport directly, constructing wire-level event shapes (EventBuilder, tags, Kind constants) above the fabric layer.

## Trigger

Merging the fabric-architecture branch which introduced Kind1Nip29Provider; the six leak sites where wire shapes were built above the new abstraction boundary had to be closed.

## Decision

provider.publish(ev, keys) is the single wire-publish entry above the seam. The session engine now takes the provider instead of codec+transport. All six leak sites (doctor probe, user_prompt, project_edit, propose, turn_end, session engine spawn) were rewired through the provider. Proposal became a first-class DomainEvent with its own codec arms.

## Consequences

- No wire-level event construction above the fabric layer (only a test oracle remains)
- Outbound inbox replies route through provider.send, so they join the original message's canonical thread — behavior master's version lacked
- project_edit uses the nip29 lifecycle module rather than raw event builders
- The rebase kept fabric's rewritten monoliths and ported each later master commit by hand rather than mechanically

## Open Tail

- The tenex-edge-fabric worktree can be pruned now that it's fully merged
- Production daemon at ~/.local/bin/tenex-edge is still the old build — deploy when ready

## Evidence

- transcript lines 120-170
- transcript lines 4400-4660
- transcript lines 4715-4722
- transcript lines 5272-5278

