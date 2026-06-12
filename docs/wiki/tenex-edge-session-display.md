---
title: Tenex-Edge Session Display
slug: tenex-edge-session-display
topic: tenex-edge
summary: Session IDs displayed in the `tenex tail` command use the hash-based `session_short_code`, matching the display format used by `who` and `send-message`
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-10
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:435ec383-d607-459b-a712-a00ed4decaa7
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-09T15-01-20-019eac42-32f0-7ff0-bda2-da2de3b78ed7
---

# Tenex-Edge Session Display

## Session Display IDs

Session IDs displayed in the `tenex tail` command use the hash-based `session_short_code`, matching the display format used by `who` and `send-message`. Session IDs are represented by a `SessionId` newtype (defined in `src/util.rs`) whose `Display` implementation is hardwired to `session_short_code`, making correct display implicit at format call sites and enforcing type safety against accidental use of pubkey truncation functions. The `short_id` function has been renamed to `pubkey_short` everywhere to make it explicit that it is only valid for pubkeys. Domain structs (`Presence::session_id`, `Mention::target_session`, `Mention::from_session`, `WhoRow::session_id`) use the `SessionId` newtype, while the DB layer uses `String`, converting at the codec/domain boundary via `.as_str()` for writes and `SessionId::from(s)` for reads.

<!-- citations: [^435ec-3] [^ab999-4] [^ab999-41] [^56f9f-3] [^56f9f-4] [^56f9f-8] [^56f9f-12] [^56f9f-14] [^435ec-5] [^56f9f-19] -->
## Thread Display in CLI

The `threads --project` CLI must print the full thread id (not a truncated `short_id`) so it can be used with `--thread`. <!-- [^ab999-42] -->

## RPC Thread Meta Response

`rpc_thread_meta` must not return bare JSON `null` for a missing thread; it must return an empty object so the client interprets it as a valid empty result rather than 'neither ok nor error'. <!-- [^ab999-43] -->

## Who Command Display Format

The `tenex-edge who` command renders the current project as a top-level heading with one line per active session in the format `$agent [session $truncated_id] - $status`, omitting the `agents:` label and `@project` suffix. When listing agents from other projects, the display format always includes `$agent@$projectSlug`; the project slug is only omitted when the agent is in the same project context. The other-project summary displays unique agent slugs per project in the format `$agent@$project`, avoiding duplicate bullets for multiple sessions of the same agent, and includes a count of other agents. When `who --all-projects` is used, every row includes the project suffix (`$agent@$project`) rather than relying on a project heading. It never prints `(this machine)`; it only appends `(remote on $machineName)` when the agent's host differs from the local host.

<!-- citations: [^rollo-11] [^rollo-21] -->
## Injected/Tooling Instructions Format

Injected/tooling instructions for agents use the format `<agent@project|session-id>` rather than the ambiguous `<agent|session-id>` form. <!-- [^rollo-12] -->

## Who Live Renderer Line Endings

The `who --live` renderer writes `\r\n` instead of `\n` in raw terminal mode so each line returns to column 0. <!-- [^rollo-13] -->
