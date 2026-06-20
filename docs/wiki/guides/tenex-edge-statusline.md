---
title: tenex-edge Statusline
slug: tenex-edge-statusline
topic: tenex-edge
summary: "The statusline format is: `claude@host [session-id] â¬¡{member_count} â{session_count} {activity} {distill_error} {inbox_segment}`, where â¬¡ is count of NIP-"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-17
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:e42f09d7-5fb0-438b-a356-216870390540
  - session:622711fa-5176-4580-b311-d66446c2924b
  - session:633f8f7f-37f8-409c-90a9-ef64b0dc3216
  - session:e4d3c252-a2ff-40fe-b18d-a608f557322b
  - session:rollout-2026-06-16T14-03-13-019ed019-80ea-7e31-aa11-7302f859b853
---

# tenex-edge Statusline

## Format

The statusline format is: `claude@host [session-id] ⬡{member_count} ◉{session_count} {activity} {distill_error} {inbox_segment}`, where ⬡ is count of NIP-29 group member agents (hidden when zero), ◉ is count of sessions with non-expired presence heartbeats (including idle), activity is the session's self-reported status or 'idle', and the inbox segment shows pending or recently-consumed mentions. The status line displays incoming messages for the current session. Session activity is shown as `✎ {status}` when working (green pencil), or `· idle` (dimmed) when idle. Status text is truncated at ~40 chars with `…`; message content at ~50 chars with `…`. When the agent is not a member of the project group but member_count > 0, the statusline shows `⚠ not in group {project}` in bold red. When a distillation error is present, the statusline renders `⚠ distill: <message>` in bold red between the status and inbox segments.

Project tabs are sorted by live session count in descending order, with alphabetical tiebreaker.

<!-- citations: [^e42f0-1] [^62271-6] [^633f8-5] [^rollo-72] -->
## Inbox Display

The statusline snapshot merges explicit chat mentions (where mentioned_session equals the current session) into the pending and recent arrays alongside direct inbox rows. Only explicit chat mentions (where mentioned_session equals this session) appear as unread in the status line, matching the existing unread count precedent, rather than all project chat rows. The statusline includes recently delivered chat mentions (after turn-start drains them), not just pending ones, to prevent the status line from dropping rows inconsistently around delivery.

Pending inbox mentions display as `✉N sender@host: subject (+k)` in bold yellow; when multiple are pending, the newest is shown with a count of the rest. Recently consumed (drained) inbox mentions display as `✉✓ sender@host: subject` dimmed, lingering for 30 seconds after delivery before disappearing. When inbox is empty, the message segment is entirely absent — no `✉0` noise.

<!-- citations: [^e42f0-2] [^rollo-73] -->
## Daemon and CLI Behavior

The statusline daemon RPC (`rpc_statusline`) is a pure-read operation with zero state.db writes, matching the turn-check pattern to avoid reintroducing concurrent writers. It reads errors newer than 5 minutes (`DISTILL_ERROR_TTL_SECS = 300`) and includes them as `distill_error` in the response. The statusline CLI must fail open: if the daemon is unreachable, it prints nothing and exits 0, never blocking or erroring. The statusline CLI uses `call_no_spawn` so it never boots a daemon just to draw a line, preventing conflict with idle-exit. The `is_member` field in the statusline response defaults to true on error, preventing false membership warnings from transient failures.

The statusline daemon RPC (`rpc_statusline`) is a pure-read operation with zero state.db writes, matching the turn-check pattern to avoid reintroducing concurrent writers. It reads errors newer than 5 minutes (`DISTILL_ERROR_TTL_SECS = 300`) and includes them as `distill_error` in the response. The statusline CLI must fail open: if the daemon is unreachable, it prints nothing and exits 0, never blocking or erroring. The statusline CLI uses `call_no_spawn` so it never boots a daemon just to draw a line, preventing conflict with idle-exit. The `is_member` field in the statusline response defaults to true on error, preventing false membership warnings from transient failures. `status_delta_since` deduplicates local session rows against matching peer echo rows by session_id, preferring the local row. A session is never told its own reported status; the `exclude` parameter filters out both the local row and any round-tripped peer echo for the viewer's own session_id. Regression tests prove that `status_delta_since` deduplicates a local session row and its matching peer echo into a single delta item, and that it excludes a session's own status even when that status has round-tripped into `peer_session_state`. <!-- [^e4d3c-1] -->

<!-- citations: [^e42f0-3] [^633f8-6] -->
## Database Schema

A `delivered_at` column on the inbox table supports the 30-second 'recently consumed' window for the statusline's inbox display. <!-- [^e42f0-4] -->

## Integration with ccstatusline

The existing `pc` statusline (specifically `pc hook statusline`) and the tenex-edge statusline both run via ccstatusline as a multi-line multiplexer — line 1 is the proactive-context line, line 2 is the fabric line. ccstatusline is installed globally via npm (not bun, which produced a dangling symlink), and the settings.json statusLine command points to `/Users/pablofernandez/.bun/bin/ccstatusline`. The ccstatusline refresh interval is set to 3 seconds. <!-- [^e42f0-5] -->
