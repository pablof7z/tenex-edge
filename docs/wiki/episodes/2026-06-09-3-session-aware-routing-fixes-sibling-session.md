---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: root-cause
status: active
subjects:
  - tenex-edge
  - message-routing
  - same-pubkey-sibling-delivery
supersedes: []
related_claims: []
source_lines:
  - 966-1013
  - 1105-1178
  - 1203-1278
captured_at: 2026-06-12T20:02:14Z
---

# Episode: Session-aware routing fixes sibling-session mention delivery

## Prior State

Mentions between sessions of the same agent (same pubkey, different sessions) were silently dropped. Sender resolution picked the latest session across all agents (agent-agnostic). Per-agent seen_mentions dedup blocked session-targeted delivery once any sibling marked an event seen.

## Trigger

Channel reply from the flagged session demonstrably fired (visible on screen) but never arrived in the sender's inbox. Controlled repro confirmed: a claude→claude sibling-session mention produces zero inbox rows and zero seen_mentions entries.

## Decision

Three fixes: (A) Local delivery on publish — when to_pubkey is a hosted key, route the event directly into the recipient session's inbox by the published event-id, idempotent on inbox PK (event_id, target_session). This replaces the broken assumption that the relay would echo self-published events back. (B) Session-targeted mentions bypass per-agent dedup; agent-wide (untargeted) mentions still dedup per-agent. (C) Agent-scoped session resolution — resolve_session honors TENEX_EDGE_AGENT, with agent-scoped latest-alive fallback.

## Consequences

- The initial diagnosis (daemon self-skips because it authors as the agent) was wrong — the real root cause was that there was NO local delivery path at all; relays don't re-deliver to the connection that published the event.
- A secondary confound: the live repro's target session was dead (alive=0), so compute_targets returned [].
- Cross-agent delivery (opencode→claude) verified intact after the fix.
- 89 tests green including new tests: sibling_session_mention_lands_in_target_via_local_delivery, session_targeted_mention_not_blocked_by_sibling_seen, agent_wide_mention_still_deduped_per_agent, latest_alive_session_is_agent_scoped.
- seen_mentions schema was NOT migrated (to avoid colliding with concurrent NIP-29 work); the bypass is logic-level only.

## Open Tail

- seen_mentions should be migrated to a per-(pubkey, session) key when NIP-29 group work settles.

## Evidence

- transcript lines 966-1013
- transcript lines 1105-1178
- transcript lines 1203-1278

