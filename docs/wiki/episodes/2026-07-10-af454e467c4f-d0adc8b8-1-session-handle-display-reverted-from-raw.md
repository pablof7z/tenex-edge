---
type: episode-card
date: 2026-07-10
session: af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/af454e46-7c4f-4182-ab2b-ebc50b1eb9ad.jsonl
salience: product
status: active
subjects:
  - session-handle-display
  - agent-identity
  - codename
supersedes: []
related_claims: []
source_lines:
  - 15-15
  - 719-719
  - 1191-1199
captured_at: 2026-07-10T11:10:34Z
---

# Episode: Session handle display reverted from raw session_id to friendly codename

## Prior State

Commit 86b2a9fd had switched the session segment of agent/session handles from the friendly codename (e.g. `willow-echo-042`) to the raw internal session_id (e.g. `te-18c0e6e30d9c3c80-0`), and modified test assertions to lock in that behavior.

## Trigger

User observed the wrong format in practice: 'we are using the wrong session id in the name -- it should be the otan-like codename, not /te-123123122'. Investigation confirmed the prior commit was a regression that overwrote codename-based assertions with raw-id assertions.

## Decision

Revert to using `friendly_short_code(session_id)` (the codename) as the session segment in all agent/session handle renderings: identity/keys.rs display_slug, fabric_context/refs.rs session_ref, who_snapshot/dormant.rs, cli/who/render/expired.rs, and three daemon RPC/resolve call sites. Raw session_id remains internal-only.

## Consequences

- All user-facing surfaces (kind:0 name, chat From labels, statusline, who listings, expired-session render, invite resolve, RPC agents list) now show the human-readable codename instead of the opaque hex session id
- Mention resolution was already codename/raw-id/prefix agnostic and required no changes
- ~7 test assertions that had been altered to expect raw ids were reverted to expect codenames
- 13 files changed; PR #344 merged into master after CI passed

## Open Tail

*(none)*

## Evidence

- transcript lines 15-15
- transcript lines 719-719
- transcript lines 1191-1199

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json`](transcripts/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json`](transcripts/raw/2026-07-10-af454e467c4f-d0adc8b8-1-session-handle-display-reverted-from-raw.json)
