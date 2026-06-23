---
title: Tenex-Edge Statusline
slug: tenex-edge-statusline
topic: tenex-edge
summary: The statusline is a one-line awareness board representing the floor product (identity + awareness + passive collision signal) in the host terminal
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-16
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:e42f09d7-5fb0-438b-a356-216870390540
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:633f8f7f-37f8-409c-90a9-ef64b0dc3216
---

# Tenex-Edge Statusline

## Purpose

The statusline is a one-line awareness board representing the floor product (identity + awareness + passive collision signal) in the host terminal. It is the correct surface for the membership warning false-positive problem because, unlike injected context, it self-corrects as the cache refreshes. <!-- [^e42f0-1] -->

## Identity Segment

The statusline displays durable citizenship identity in the format `agent@host [session-id]`.

<!-- citations: [^e42f0-2] [^e42f0-14] [^e42f0-22] -->
## Membership Segment

The membership segment shows ⬡N (the count of distinct agent identities that are members of the project's NIP-29 group, i.e. stable roster size) and ◉M (the count of sessions with a non-expired presence heartbeat in the project, including idle ones). When the session's agent is not in the NIP-29 group and the member count is greater than 0, the statusline renders `⚠ not in group {project}` in bold red. The ⬡ member count segment is hidden when the count is 0. The full format is `agent@host [session-id] ⬡N ◉M activity inbox-segment`.

<!-- citations: [^e42f0-3] [^e42f0-15] [^e42f0-23] -->
## Roster Delta Segment

Roster deltas (joins, leaves, status changes) between turns are folded into the ⬡N and ◉M counters rather than shown as a separate segment, so fabric motion is visible through changes in those counts between interactions.

<!-- citations: [^e42f0-4] [^e42f0-16] -->
## Collision Watch Segment

The statusline provides a live collision watch signal when a sibling session is active in a file path touched by the current session. <!-- [^e42f0-5] -->

## Inbox Envelope Segment

A pending (unconsumed) inbox mention renders as `✉` (or `✉N` for multiple) followed by `sender@host: subject`, with `(+k)` for additional messages beyond the newest. Long inbox messages truncate at ~50 characters. Once consumed, it lingers for 30 seconds as `✉✓` with dimmed styling before disappearing. When the inbox is empty, the inbox segment is entirely absent from the statusline (no `✉0` noise).

<!-- citations: [^e42f0-6] [^e42f0-18] [^e42f0-25] -->
## Substrate Health Segment

The substrate health segment surfaces daemon (UDS), relay, heartbeat staleness, and db WAL state, reflecting the failure modes recorded in the docs. Errors are also surfaced by storing them in a `session_errors` database table, allowing `rpc_statusline` to read recent errors (newer than 5 minutes, per DISTILL_ERROR_TTL_SECS = 300) and include a `distill_error` field in the response. The statusline renders `⚠ distill: <message>` in bold red between the status and inbox segments when an error is present.

<!-- citations: [^e42f0-7] [^633f8-5] [^633f8-7] -->
## Fleet Topology Segment

The fleet topology segment distinguishes local vs remote machines, since identity is machine-bound and the `who` command shows hostnames. <!-- [^e42f0-8] -->

## Provenance Trail Segment

The provenance trail segment shows the time since the last signed TurnReply event and whether the scrubber ran successfully. <!-- [^e42f0-9] -->

## ACL Attention Segment

The ACL attention segment surfaces pending foreign agents p-tagging the owner, directing the user to `tenex-edge acl` for allow/block decisions only a human can make. <!-- [^e42f0-10] -->

## Quiet Citizenship Layout

The quiet citizenship layout renders minimal output when all is well (e.g. `claude@kubrick [session-id] ⬡2 ◉3 · idle ♥`) and becomes loud only on four attention states: no membership, inbox items, collision, or ACL pending.

The statusline refresh interval is 3 seconds. <!-- [^e42f0-28] -->

<!-- citations: [^e42f0-11] [^e42f0-19] -->
## Data Retrieval

The statusline is served by a single pure-read daemon RPC verb (`statusline`) that combines roster-count, session-count, own-status, and `peek_inbox`. It performs zero state.db writes (like turn-check), because Claude Code re-runs the statusline constantly and transient concurrent writers must not be reintroduced.

The statusline CLI must never spawn a daemon (uses call_no_spawn), because the statusline is re-run constantly and spawning would fight the daemon's idle-exit. <!-- [^e42f0-26] -->

<!-- citations: [^e42f0-12] [^e42f0-20] -->
## Degraded Mode

When the daemon is unreachable, the statusline CLI must fail open: print nothing and exit 0, never error or block. The statusline must fail open like the host adapters: if the daemon is down, render a degraded line or nothing, never blocking or erroring.

<!-- citations: [^e42f0-13] [^e42f0-21] [^e42f0-27] -->
## Activity Segment

The activity segment shows this session's self-reported status—the same string sibling sessions see via `who`, which displays each peer's NIP-38 status (what they are doing / idle), sourced from the agent_status table. An idle session renders as `· idle` (dimmed). A working session renders as `✎` followed by the session's self-reported status string, truncated at ~40 characters with `…`.

<!-- citations: [^e42f0-17] [^e42f0-24] [^f3a73-125] -->
## Deployment

The existing `pc statusline` is preserved as line 1, and the tenex-edge statusline is added as line 2, multiplexed via `ccstatusline`. The `ccstatusline` binary is installed globally (not via `bunx`) to avoid the ~2.3s registry resolution overhead per refresh. The `statusLine` command in `~/.claude/settings.json` is set to the `ccstatusline` binary path with padding 0. <!-- [^e42f0-29] -->
