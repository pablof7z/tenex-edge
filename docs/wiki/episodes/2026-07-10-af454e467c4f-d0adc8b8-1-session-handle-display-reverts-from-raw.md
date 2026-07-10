---
type: episode-card
date: 2026-07-10
session: af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/af454e46-7c4f-4182-ab2b-ebc50b1eb9ad.jsonl
salience: reversal
status: active
subjects:
  - agent-session-handle
  - display-slug
  - friendly-short-code
supersedes:
  - 2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-switched-from-raw
related_claims: []
source_lines:
  - 15-15
  - 719-719
  - 1189-1198
captured_at: 2026-07-10T10:44:37Z
---

# Episode: Session handle display reverts from raw session_id to friendly codename

## Prior State

Commit 86b2a9fd ('Use agent session handles for agent profiles') had switched the session segment of the @agent/session display handle from the friendly codename (e.g. 'lark-summit-042') to the raw internal session_id (e.g. 'te-18c0e6e30d9c3c80-0'). Test assertions were edited to lock in this behavior, despite test names like 'session_identity_agent_ref_names_pubkey_by_codename' indicating the codename was the original intent.

## Trigger

User noticed the wrong identifier in agent-facing display names and directed: 'we are using the wrong session id in the name -- it should be the otan-like codename, not /te-123123122' (line 15).

## Decision

Revert the session segment of every @agent/session display handle to use friendly_short_code(session_id) (the codename) instead of the raw session_id. Applied uniformly across all rendering sites: identity/keys.rs::display_slug(), fabric_context/refs.rs::session_ref() (both legacy and pure assemble paths), who_snapshot/dormant.rs, cli/who/render/expired.rs, and three daemon RPC/resolve call sites (invite_rpc/resolve.rs, management_command/sessions.rs, rpc/agents.rs).

## Consequences

- All user-facing session handles now consistently show @agent/codename instead of @agent/raw-session-id across kind:0 profiles, chat From labels, statusline, who listings, expired-session listings, fabric context member rendering, and RPC responses.
- Mention resolution was already flexible (accepts codename, short code, raw id, or prefix) so no breakage occurred on the resolution side.
- ~7 test assertions that had been changed to expect raw session_id were reverted to expect the codename.
- Full lib test suite (923 tests) passes; clippy clean; integration tests unaffected.
- The codename field in ExpiredSessionRow, previously computed but ignored in favor of raw session_id, is now the active display value.

## Open Tail

- Changes are not yet committed — user was asked whether to stage them.

## Evidence

- transcript lines 15-15
- transcript lines 719-719
- transcript lines 1189-1198

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverts-from-raw.json`](transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverts-from-raw.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverts-from-raw.json`](transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverts-from-raw.json)
