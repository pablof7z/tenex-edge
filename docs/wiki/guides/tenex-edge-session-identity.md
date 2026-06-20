---
title: tenex-edge Session Identity
slug: tenex-edge-session-identity
topic: tenex-edge
summary: Session IDs display as a human-friendly codename (`session_codename`) — a NATO phonetic word plus four digits (e.g. `bravo4217`) — rather than UUID prefix truncation (`short_id`) or the older 6-char hex hash
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-17
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:435ec383-d607-459b-a712-a00ed4decaa7
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
  - session:ea5dd578-ca5d-4f31-8427-3a253dd735e8
  - session:rollout-2026-06-10T22-36-00-019eb308-d484-77d2-a8ee-03f5a676ed99
  - session:rollout-2026-06-16T12-40-33-019ecfcd-d47b-7992-998f-75432d8ac4cf
  - session:rollout-2026-06-16T17-43-45-019ed0e3-68e5-7091-899d-6a4e0fcb5716
  - session:rollout-2026-06-17T10-51-45-019ed490-9414-75c3-ab93-66265458c6e9
  - session:ses_13089dfceffeDFSl8v4Lv8hCBt
---

# tenex-edge Session Identity

## Session Identity Display

Session IDs display as a human-friendly **codename** (`session_codename`) — a NATO phonetic word plus a four-digit number, e.g. `bravo4217` or `echo0163` — rather than UUID prefix truncation (`short_id`) or the older 6-character hex hash. The codename is deterministic (same session id → same codename) and copy-pasteable into `send --to`. A `SessionId` newtype in `src/util.rs` has its `Display` impl hardwired to `session_codename`, so any `{session_id}` in a format string automatically produces the codename. The codename space is 26×10000 = 260000, ample for the handful of live sessions a fabric holds but NOT collision-free at scale — it is a display/addressing convenience, never the identity (the canonical session id remains authoritative). The `short_id` utility is renamed to `pubkey_short` everywhere, making it explicit at every call site that it is for pubkeys only.

The `d` tag for kind 30315 events must be formatted as `tenex-edge:<bare-session-id>`, with no embedded JSON object in the identifier string. The `session-id` tag must contain only the bare session ID string, not a stringified JSON object. <!-- [^ea5dd-2] -->

Session IDs are local daemon state written locally first, then published to the relay as Status events (kind 30315). Remote agents' session IDs are learned by hydrating local store from their relay Status events through the materializer, which writes peer_sessions and session_status. Local sessions are authoritative locally and are not re-hydrated from the local node's own relay echo. <!-- [^rollo-57] -->

A NIP-40 expiration tag is applied to the kind:30315 heartbeat, reversing the old never-expire behavior, making liveness equal to event freshness. <!-- [^rollo-81] -->

The daemon mints a stable canonical session_id; harness ids become session_aliases, and alias-aware lookups apply everywhere they matter (routing, resume, who). <!-- [^rollo-82] -->

Hooks reassert the canonical session id, but turn/session RPCs still mutate using the raw hook id, causing canonical session_state transitions to no-op. <!-- [^rollo-83] -->

Alias fallback ignores namespace. <!-- [^rollo-84] -->

Heartbeats update the local DB only and never republish kind:30315, so NIP-40 expiration is not re-armed. <!-- [^rollo-85] -->

Session/project filters must consistently accept project slug, canonical session ID, harness alias, and codename. The `tmux resume` and `inbox send --to-session` recipient resolvers both accept a codename as a lookup path (case-insensitive, first match wins), so a user can copy `[session bravo4217]` straight from `who`/the TUI.

The context injection must tell the agent its own identity (slug and session codename), not just list other agents. The first-turn intro in `turn.rs` must read `You are {slug} [session {codename}] on the tenex-edge fabric.` instead of the previous `You are connected to the tenex-edge agent fabric`. The codename shown in the self-identity line must match the codenames displayed in the `who` output, so the agent can identify which row in the fabric list is itself.

<!-- citations: [^ea5dd-2] [^rollo-57] [^rollo-81] [^rollo-82] [^rollo-83] [^rollo-84] [^rollo-85] [^435ec-4] [^56f9f-7] [^a0037-5] [^rollo-111] [^ses_1-26] -->
## SessionId Newtype and Domain Usage

The `Presence::session_id`, `Mention::target_session`, `Mention::from_session`, and `WhoRow::session_id` domain struct fields use the `SessionId` newtype instead of `String`. DB layer fields remain `String`, with conversion to/from `SessionId` at the codec/domain boundary via `.as_str()` writes and `SessionId::from(s)` reads.

Session-state mutation uses versioned transition methods (state_version/turn_id) so that a stale distill or duplicate runtime task cannot apply.

Turn start and turn end are called by both RPC and runtime, causing double turn_id, state_version, and outbox publishes once canonical ids flow through.

turn_check writes turn_state.last_check_at, violating the pure-read requirement.

Outbox rows publish the current snapshot rather than their own version.

Who renderer tests must derive expected session display strings dynamically using `session_codename(...)` instead of hard-coded strings like `local-se`.

<!-- citations: [^56f9f-8] [^rollo-40] [^rollo-86] -->
## Corrupted Session Handling

Existing corrupted sessions (with JSON-string session IDs) persist until their opencode processes restart. <!-- [^ea5dd-3] -->

## Session Start Codename Distribution

The `session_start` daemon RPC returns JSON containing both `session_id` and `codename` fields, so callers get the codename without recomputing it. For `generates_sid` hosts (opencode), the session-start hook in `hooks.rs` prints JSON with both `session_id` and `codename` instead of just the raw session ID string. The opencode plugin only needs `session_id` from this response — the self-identity line (slug + codename) is assembled by the Rust hook and arrives in the turn-start context, so the plugin does not render the codename itself. <!-- [^ses_1-27] -->
