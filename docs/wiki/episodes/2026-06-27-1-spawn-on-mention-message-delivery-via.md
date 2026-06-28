---
type: episode-card
date: 2026-06-27
session: 01b0d1e8-aa18-4086-acad-237611e4c63d
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/01b0d1e8-aa18-4086-acad-237611e4c63d.jsonl
salience: root-cause
status: active
subjects:
  - subscription-registry
  - spawn-on-mention
  - session-startup
  - inbound-message-routing
supersedes: []
related_claims: []
source_lines:
  - 2571-2592
  - 2624-2660
  - 2661-2714
  - 2798-2869
captured_at: 2026-06-27T21:42:55Z
---

# Episode: spawn-on-mention message delivery via conditional relay replay

## Prior State

Master's `ensure_subscription` unconditionally re-subscribed each session, which triggered relay replay of stored chat via NIP-01 subscribe semantics. Spawned-on-mention sessions received their triggering kind:9 messages as a side effect of this unconditional replay.

## Trigger

Test failure: `operator_kind9_to_offline_local_agent_spawns_and_injects` — spawned agent never receives the kind:9 that summoned it. Root-cause investigation found #47's incremental `add_channel` returns empty when channel is already covered (it always is for spawned sessions, since the project's first session already subscribed it). No re-subscribe means no replay, breaking the implicit dependency on replay for spawn-on-mention delivery.

## Decision

Implement `SubscriptionRegistry::covers_channel` query + `replay_channel_chat` API. When a session becomes alive in an already-covered channel, re-apply the narrow `#h` REQ — re-applying a subscription ID replaces it in place, triggering relay replay. Gate the replay on `covers_channel` to avoid overhead for brand-new channels (which already stream their full backlog when subscribed).

## Consequences

- Spawned sessions now reliably receive their triggering kind:9 messages, completing the spawn-on-mention contract
- Subscription model now explicitly manages replay semantics (via `replay_channel_chat`) instead of relying on side effects of unconditional re-subscribe
- Zero overhead for new-channel sessions; only already-covered channels incur a narrow replay on session birth
- Feature's incremental subscription strategy now has well-defined recovery path for sessions born into pre-existing channels

## Open Tail

*(none)*

## Evidence

- transcript lines 2571-2592
- transcript lines 2624-2660
- transcript lines 2661-2714
- transcript lines 2798-2869

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-27-1-spawn-on-mention-message-delivery-via.json`](transcripts/2026-06-27-1-spawn-on-mention-message-delivery-via.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-27-1-spawn-on-mention-message-delivery-via.json`](transcripts/raw/2026-06-27-1-spawn-on-mention-message-delivery-via.json)
