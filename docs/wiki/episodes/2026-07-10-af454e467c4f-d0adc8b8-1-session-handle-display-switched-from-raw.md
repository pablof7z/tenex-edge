---
type: episode-card
date: 2026-07-10
session: af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/af454e46-7c4f-4182-ab2b-ebc50b1eb9ad.jsonl
salience: product
status: superseded
subjects:
  - session-handle-display
  - friendly-short-code
  - agent-identity
supersedes: []
related_claims: []
source_lines:
  - 15-15
  - 25-26
  - 178-191
  - 232-237
captured_at: 2026-07-10T10:32:56Z
---

# Episode: Session handle display switched from raw session id to codename

## Prior State

Agent session handles and display names used the raw session id (e.g., `te-123123122`) as the session segment in `@agent/session` references and kind:0 profile names. A `friendly_short_code` codename generator existed but was only used in `expired_sessions.rs`, not in the live session handle path.

## Trigger

User correction at line 15: 'we are using the wrong session id in the name -- it should be the otan-like codename, not `/te-123123122`'

## Decision

Session handles and agent-facing display names should use the `friendly_short_code` codename (two-word deterministic name from CODE_WORDS_A/CODE_WORDS_B) instead of the raw `te-{hex}` session id as the session segment of `@agent/session`.

## Consequences

- All renderers that format `@agent/session` must route the session segment through `friendly_short_code` rather than passing the raw session id
- The `SessionIdentity` already carries a `codename` field (set via `friendly_short_code`), so `display_slug` and `session_handle` should consume it instead of the raw `session_id`
- kind:0 profile names, `who` output, fabric context member rows, status lines, and p-tag mentions all need to consistently show the codename
- The codename must remain round-trippable — the daemon must resolve a codename prefix back to the full session for `channel read --id` and p-tag mentions

## Open Tail

- Verify that all call sites of `session_handle` and `display_slug` pass the codename, not the raw session id
- Ensure prefix resolution from codename back to full session id still works with the short-code mapping

## Evidence

- transcript lines 15-15
- transcript lines 25-26
- transcript lines 178-191
- transcript lines 232-237

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-switched-from-raw.json`](transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-switched-from-raw.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-switched-from-raw.json`](transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-switched-from-raw.json)
