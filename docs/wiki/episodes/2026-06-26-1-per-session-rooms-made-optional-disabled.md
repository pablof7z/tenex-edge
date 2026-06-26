---
type: episode-card
date: 2026-06-26
session: 8510c3cc-9722-47a4-90ee-2f489646f5b8
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/8510c3cc-9722-47a4-90ee-2f489646f5b8.jsonl
salience: product
status: active
subjects:
  - per-session-rooms
  - session-channel-selection
supersedes: []
related_claims: []
source_lines:
  - 1-7
  - 348-360
  - 393-403
  - 544-608
  - 612-641
  - 754-779
captured_at: 2026-06-26T20:25:26Z
---

# Episode: Per-session rooms made optional, disabled by default

## Prior State

Sessions without explicit channel override always minted a fresh per-session NIP-29 subgroup; both launch and non-launch paths created rooms automatically

## Trigger

User directive: 'let's make it optional to have these per-session rooms' with three specified behaviors: (1) when disabled, tenex-edge launch opens channel picker TUI, (2) when disabled and not using launch with no channel, use project channel, (3) when enabled, preserve legacy per-session mint behavior

## Decision

Added `perSessionRooms: bool` config flag in ~/.tenex-edge/config.json (defaults false). Updated decide_session_room() to accept the flag and return UseExisting{work_root} when disabled (vs Mint{work_root} when enabled). Launch path opens interactive channel picker when flag disabled and --channel omitted; non-TTY contexts bail loudly requesting explicit --channel <id>

## Consequences

- Breaking change: default behavior inverts from always-mint to always-project; existing deployments must opt into per-session rooms via config
- Launch and non-launch code paths now converge on the same channel-picker trigger (disabled + no --channel)
- Non-TTY automation loses silent fallback; must now pass --channel <id> explicitly when disabled
- All integration tests exercising per-session-room feature must opt into perSessionRooms:true to maintain test intent
- is_session_room() safety preserved (already keys off project id in store, so project channel correctly remains a non-room)

## Open Tail

*(none)*

## Evidence

- transcript lines 1-7
- transcript lines 348-360
- transcript lines 393-403
- transcript lines 544-608
- transcript lines 612-641
- transcript lines 754-779

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-1-per-session-rooms-made-optional-disabled.json`](transcripts/2026-06-26-1-per-session-rooms-made-optional-disabled.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-1-per-session-rooms-made-optional-disabled.json`](transcripts/raw/2026-06-26-1-per-session-rooms-made-optional-disabled.json)
