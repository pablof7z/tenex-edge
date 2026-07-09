---
type: episode-card
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
salience: root-cause
status: active
subjects:
  - per-session-keys
  - reply-envelope
  - shared-pubkey-contradiction
  - daemon-design
supersedes:
  - 2026-07-09-b70718e17221-cb4e41ba-3-per-session-key-model-inverts-shared
related_claims: []
source_lines:
  - 4000-4005
  - 4032-4038
  - 4139-4158
captured_at: 2026-07-09T18:00:57Z
---

# Episode: Per-session pubkey model inverts shared-pubkey design rationale in docs

## Prior State

daemon-design.md §8a/§8b and reply-envelope rationale in fabric-architecture.md asserted: "identity is (agent, machine) → sibling sessions of one agent share a pubkey, so the author key alone can't address a reply." This justified messages.author_session as a canonical column and specific reply-addressing logic.

## Trigger

During the docs reconciliation pass (#318), the docs agent verified against current code and found the redesign inverted the model: each session now mints its own key, so the pubkey DOES uniquely address a session — making the shared-pubkey rationale factually backwards.

## Decision

Tracked as issue #322 for a dedicated, code-verified rewrite rather than fixing blind in a docs pass. The docs agent and assistant explicitly scoped it out as too risky to rewrite without verifying how reply-addressing and messages.author_session actually work in current code.

## Consequences

- Known tension: daemon-rpc-surface.md now says per-session key/codename while daemon-design.md §8a still says shared per-agent pubkey — docs are internally contradictory
- The reply-envelope rationale (messages.author_session as canonical column, reply-degradation logic) may be partially or fully obsolete under per-session keys — requires code verification to determine
- Rewriting correctly requires understanding current routing/reply code, not just docs editing — doing it blind would make docs more wrong
- Filed as #322: a code-informed design-doc revision, not a terminology pass

## Open Tail

- #322 remains open — needs code-verified rewrite of daemon-design §8a/§8b and reply-envelope passages in fabric-architecture.md and fabric-architecture-implementation.md
- Must determine whether messages.author_session and reply-degradation logic are still needed or can be simplified under per-session keys

## Evidence

- transcript lines 4000-4005
- transcript lines 4032-4038
- transcript lines 4139-4158

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-b70718e17221-cb4e41ba-2-per-session-pubkey-model-inverts-shared.json`](transcripts/2026-07-09-b70718e17221-cb4e41ba-2-per-session-pubkey-model-inverts-shared.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-2-per-session-pubkey-model-inverts-shared.json`](transcripts/raw/2026-07-09-b70718e17221-cb4e41ba-2-per-session-pubkey-model-inverts-shared.json)
