---
type: episode-card
date: 2026-07-10
session: af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/af454e46-7c4f-4182-ab2b-ebc50b1eb9ad.jsonl
salience: reversal
status: active
subjects:
  - session-handle-display
  - agent-identity
  - friendly-short-code
supersedes:
  - 2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverts-from-raw
related_claims: []
source_lines:
  - 15-16
  - 719-719
  - 1189-1199
  - 1362-1368
captured_at: 2026-07-10T14:11:01Z
---

# Episode: Session handle display reverted from raw session_id to friendly codename

## Prior State

Commit 86b2a9fd ('Use agent session handles for agent profiles') had switched the session segment of all `@agent/session` display handles from the friendly codename (e.g. `lark-summit-042`) to the raw internal session id (e.g. `te-18c0e6e30d9c3c80-0`). This affected every user-visible surface: kind:0 profile names, chat From labels, statusline, `who` output, expired-session listings, fabric-context member rows, and daemon RPC/resolve call sites. Test assertions were rewritten to lock in the raw-id behavior.

## Trigger

User noticed the wrong identifier appearing in agent names — 'it should be the otan-like codename, not `/te-123123122`' (line 15). Root-cause analysis confirmed that 86b2a9fd deliberately replaced codename with raw session_id, and that a `friendly_short_code`/codename mechanism already existed for exactly this purpose.

## Decision

All session handle rendering now uses `friendly_short_code(session_id)` (the codename) instead of the raw `session_id`. The fix was applied at the root source — `SessionIdentity::display_slug()` — plus six downstream render/resolve sites: `fabric_context/refs.rs::session_ref()`, `who_snapshot/dormant.rs`, `cli/who/render/expired.rs`, and three daemon RPC/resolve paths (`invite_rpc/resolve.rs`, `management_command/sessions.rs`, `rpc/agents.rs`). ~7 test assertions reverted to expect the codename.

## Consequences

- User-visible handles like `@coder/lark-summit-042` are restored across all surfaces (kind:0 names, who output, fabric context member rows, expired-session listings, invite/resolve paths).
- Mention resolution is unaffected — it already accepted codename, short code, raw id, or prefix, so no input-parsing changes were needed.
- The legacy `codename` field in ExpiredSessionRow and SessionClaim, which was previously computed but ignored in display, is now the authoritative display value.
- PR #344 merged into master, making the fix live upstream.

## Open Tail

*(none)*

## Evidence

- transcript lines 15-16
- transcript lines 719-719
- transcript lines 1189-1199
- transcript lines 1362-1368

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json`](transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json`](transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json)
