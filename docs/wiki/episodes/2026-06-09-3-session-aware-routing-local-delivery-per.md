---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: root-cause
status: active
subjects:
  - tenex-edge-routing
  - sibling-session-delivery
  - mention-dedup
  - sender-resolution
supersedes: []
related_claims: []
source_lines:
  - 966-1013
  - 1105-1122
  - 1151-1180
  - 1203-1253
captured_at: 2026-06-17T23:48:53Z
---

# Episode: Session-aware routing: local delivery, per-session dedup, agent-scoped resolution

## Prior State

Three routing bugs: (A) same-machine sibling-session mentions were dropped — daemon only published to relay and relied on echo-back, but relays don't re-deliver to the publishing connection; (B) seen_mentions dedup was per-agent (pubkey only), so once any sibling marked an event seen, the intended target session was blocked; (C) resolve_session fell back to latest_alive_session_for_project agent-agnostically, so a claude send was signed/recorded as opencode when opencode was the newest session.

## Trigger

Channel test revealed the reply path was broken — a claude→claude sibling-session mention never arrived in the inbox. Empirical repro confirmed: claude→claude produced zero inbox rows, while opencode→claude (cross-agent) worked fine. Initial self-skip hypothesis was disproven (Mention arm has no is_self guard); real cause was the missing local-delivery path.

## Decision

Three fixes: (A) synchronous local delivery in rpc_send_message — when to_pubkey ∈ hosted_pubkeys(), route the just-published event into the recipient's inbox by published EventId (idempotent on inbox PK (event_id, target_session), so relay echo cannot double-deliver); (B) session-targeted mentions bypass per-agent is_mention_seen (agent-wide mentions still dedup per-agent, preserving don't-resurface-in-every-session); (C) resolve_session now accepts agent: Option<&str> and uses Store::latest_alive_session_for_agent_in_project, with TENEX_EDGE_AGENT threaded through all relevant CLI verbs.

## Consequences

- Live before/after proof: claudeA→claudeB mention lands in B's inbox (0 rows in sender A's inbox); cross-agent opencode→claude still works
- 89 tests green (up from ~81), including new tests: sibling_session_mention_lands_in_target_not_sender, session_targeted_mention_not_blocked_by_sibling_seen, local_delivery_by_event_id_is_idempotent_and_targets_sibling, agent_wide_mention_still_deduped_per_agent, latest_alive_session_is_agent_scoped, daemon_integration sibling-session-via-local-delivery
- Test isolation fix: daemon_integration tests now env_remove TENEX_EDGE_AGENT/SESSION to prevent live-shell env leak
- The per-agent dedup bypass is a code-level bypass (not a schema migration) to avoid colliding with concurrent NIP-29 rewrite of state.rs — the docs already claimed per-(pubkey, session) dedup

## Open Tail

*(none)*

## Evidence

- transcript lines 966-1013
- transcript lines 1105-1122
- transcript lines 1151-1180
- transcript lines 1203-1253

