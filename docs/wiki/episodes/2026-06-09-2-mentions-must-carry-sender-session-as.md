---
type: episode-card
date: 2026-06-09
session: d208c058-7b2b-4ff8-bb82-d63623d51097
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d208c058-7b2b-4ff8-bb82-d63623d51097.jsonl
salience: product
status: active
subjects:
  - mention
  - from-session
  - return-envelope
  - inbox
  - wire-protocol
supersedes: []
related_claims: []
source_lines:
  - 806-824
  - 1120-1567
  - 1568-1673
captured_at: 2026-06-17T23:57:01Z
---

# Episode: Mentions must carry sender session as return envelope

## Prior State

Injected mentions identified the sender only by slug@project — no session id, so the recipient could not address a reply to the originating session

## Trigger

User observed that the injected message format showed 'mention from claude@tenex-edge' with no session to reply to, making inter-agent replies unrouteable

## Decision

Thread sender session end-to-end: new `from_session: Option<String>` on the Mention domain type → `from-session` wire tag in codec → `from_session` column in InboxRow/inbox schema → derived reply-to handle in all injection/display sites (session prefix if resolvable, else slug@project)

## Consequences

- Wire protocol extended with `from-session` tag; back-compatible: old events without the tag decode to None
- Schema migration (ALTER TABLE) adds `from_session` column to existing state.db
- Five previously duplicated formatting sites consolidated into two shared helpers (mention_reply_handle, format_mention_line)
- Fabric-architecture doc updated: inbox entity table now lists from_session as 'reply address'; canonical messages schema gains author_session; SendIntent gains from_session; dual-write backfill rule explicitly copies inbox.from_session → messages.author_session
- MCP channel injection path identified as a separate path not yet covered by this change

## Open Tail

- MCP reply_to channel tag needs equivalent from_session treatment
- Daemon restart required for migration to take effect on running state.db

## Evidence

- transcript lines 806-824
- transcript lines 1120-1567
- transcript lines 1568-1673

