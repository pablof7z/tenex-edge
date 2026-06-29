---
title: Tenex-Edge Agent Identity
slug: tenex-edge-agent-identity
topic: tenex-edge
summary: Sessions carry one authoritative agent-instance identity from creation
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12ce-2569-72e0-b959-6d87d5daec5d
  - session:bd8689c8-4a5f-45b3-9dbe-758baec2a2f4
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
---

# Tenex-Edge Agent Identity

## Agent-Instance Identity

Sessions carry one authoritative agent-instance identity from creation. AgentInstance is the single authoritative identity type owning base-vs-ordinal identity policy, carrying base_slug, base_pubkey, ordinal, and selected pubkey; it is created at session birth, threaded through EngineParams, and projected from the store for read-side callers. AgentInstance provides display_slug(), agent_ref(), and signing_keys(&base_keys) as the only methods for base-vs-ordinal identity policy; display_slug() returns the ordinal label (e.g. 'haiku' for ordinal 0, 'haiku1' for ordinal 1). EngineParams carries the AgentInstance plus base keys, replacing the prior five parallel slug/pubkey/key fields it previously held. Store::instance_identity_for_session() and DaemonState::session_instance(&Session) are the projection helpers that read-side callers use to obtain the authoritative AgentInstance, with safe fallback. Every publisher, renderer, and router (status, chat, profile, who, statusline, heartbeat, proposal, mention routing) consumes the AgentInstance rather than reconstructing identity policy at edge sites. Each agent session signs its own kind:0 profile event with its own key: the ordinal key when it exists, the base key for ordinal 0, never the base key for all sessions. Two concurrent sessions of the same base agent publish distinct, self-consistent identities (claude for ordinal 0 and claude1 for ordinal 1, each with its own pubkey) on kind:0 and `who`, verified by the concurrent_same_agent_sessions_publish_consistent_identities integration test. (Previously: the ad-hoc keys_for_session(..).unwrap_or(base) and base-slug-with-ordinal-pubkey edge policy was used instead of AgentInstance.)

<!-- citations: [^019f1-3d443] [^bd868-ed09a] [^bd868-323a9] [^bd868-4f5d3] -->
## Naming Conventions

Agent names use the format agentName{ordinal} (e.g., 'haiku1'). Duplicate agent names must not appear in the same channel. The ordinal instance label (e.g. haiku1) is the primary same-agent disambiguator on the wire.

<!-- citations: [^019f1-d73ec] [^bd868-96ae0] -->
## Session Identification

Local sessions use agent-instance labels (e.g., 'haiku1') for p-tagging. Raw session_id is the only internal correlation id. SessionId's Display impl renders the raw session_id.

<!-- citations: [^019f1-10a4e] [^bd868-d7a8c] [^bd868-55c14] -->
## Identity Commands

Self-identity output is folded into the tenex-edge who command's agent-context output. When run by an agent, who displays a self header: 'You are **{label}** on **{channel}** ({host}).' followed by pubkey, status, member, and pending. The who self block carries label, pubkey, channel, host, is_member, working, status, pending, created_at, and a raw session_id. The who fabric block's (you) member match keys on the ordinal-selected pubkey via instance.pubkey rather than rec.agent_pubkey. Concurrent same-agent instances render their ordinal slugs directly (e.g. haiku/haiku1).

<!-- citations: [^019f1-f1f53] [^bd868-16873] [^bd868-6a26a] [^bd868-9b5dc] -->
## Chat Mention Resolution

Chat mention extraction in extract_mentions accepts agent-slug-shaped tokens ([A-Za-z0-9._-]+, optionally @host-qualified). `@<agent-label>` targeting (e.g. @haiku / @haiku1) resolves to the selected instance pubkey. resolve_agent_pubkey resolves an agent-instance label (e.g. haiku1) to the ordinal-selected pubkey by reverse-looking-up relay_profiles by slug and host. Mention resolution never blocks chat delivery: an unresolvable mention token is silently treated as no-mention rather than erroring. The chat write CLI confirmation line displays 'mentioning @{label}' using the resolved agent label, falling back to plain 'sent chat {id}' when there is no mention.

<!-- citations: [^bd868-17c6e] [^bd868-b43e5] -->
## Session Resume

tmux resume resolves sessions by exact raw session_id then session_id prefix only. The Resume CLI help text reads 'Session id (prefix) to resume.' The HookTail CLI help text reads 'Filter panes/events to a session id (or prefix)'.

<!-- citations: [^bd868-7dcbb] [^bd868-eafc6] -->
## Operator Surface and Logging

The TUI session row tag surfaces the raw session_id or agent-instance label. Tail session tags use the raw session_id for operator correlation. The slog per-session debug log filename uses the raw session_id. The session_label function in idref.rs renders 'agent@host' (degrading to raw session_id only when slug is empty). The hook echo-session-id response emits only {"session_id": canonical}. Session-start hook responses and logs do not expose extra display aliases.

<!-- citations: [^bd868-0f2b4] [^bd868-43382] -->
## Documentation Updates

The README, daemon-rpc-surface, and tail-v2-design docs reflect `@<agent-label>` targeting and raw session ids as internal correlation handles.

<!-- citations: [^bd868-1e433] [^bd868-16cba] -->
## Skill Reference

The `tenex-edge-identity-routing` skill teaches the current identity model: durable ordinal identities, `AgentInstance`, base vs ordinal keys, agent-label routing, and raw session IDs as internal-only handles. <!-- [^019f1-9229d] -->
