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
updated: 2026-06-15
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:435ec383-d607-459b-a712-a00ed4decaa7
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-09T15-01-20-019eac42-32f0-7ff0-bda2-da2de3b78ed7
  - session:215d979a-a054-4e2b-b349-851e0d874d6d
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
---

# Tenex-Edge Session Display

## Session Display IDs

Session IDs displayed in the `tenex tail` command use the hash-based `session_short_code` (rather than a UUID-based prefix), matching the display format used by `who` and `send-message` for consistency across tail, who, and send-message commands. Session IDs are represented by a `SessionId` newtype (defined in `src/util.rs`) whose `Display` implementation is hardwired to `session_short_code`, making correct display implicit at format call sites (format strings with {session_id} are automatically correct) and compile-time enforcing type safety against accidental use of pubkey truncation functions (`pubkey_short`). The `short_id` function has been renamed to `pubkey_short` everywhere to make it explicit that it is only valid for pubkeys, preventing accidental use on session IDs. Domain structs (`Presence::session_id`, `Mention::target_session`, `Mention::from_session`, `WhoRow::session_id`) use the `SessionId` newtype throughout the domain layer, while the DB layer uses `String`, converting at the codec/domain boundary via `.as_str()` for writes and `SessionId::from(s)` for reads.

The PostToolUse awareness delta output displays session IDs as the 6-character short code (matching `who` and tmux display), not raw UUIDs, so they are copy-pasteable into `send --to`. <!-- [^a0037-6] -->

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

## Exited Sessions Panel

The exited sessions panel filters and displays past sessions from the past X hours. The default time filter is 4 hours. Users can change the number of hours using the + and - keys. The [e] key toggles the exited sessions panel on and off. The [+] or [=] keys increase the hours filter in steps of +1h up to 12h, +6h up to 48h, and +24h beyond that. The [-] key decreases the hours filter in the reverse step pattern, with a minimum of 1h. The section header displays the active time window, e.g. 'Exited sessions (past 4h)'. The help line displays '[e] hide exited  [-/+] 4h' when the exited panel is visible, and '[e] show exited' when hidden. <!-- [^215d9-1] -->
