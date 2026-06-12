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
  - inbox
  - reply-address
supersedes: []
related_claims: []
source_lines:
  - 806-1567
captured_at: 2026-06-12T20:12:01Z
---

# Episode: Mention return envelope — from_session threaded end-to-end

## Prior State

Injected mentions showed only [mention from slug@project] with no sender session. A recipient could not determine which session of the sending agent to reply to — the author pubkey alone is insufficient because sibling sessions share it.

## Trigger

User observed the injected message format: 'right now we're sending [mention from claude@tenex-edge] … — no session id to know who it came from'.

## Decision

Add from_session: Option<String> end-to-end: domain type (Mention struct), wire tag (from-session), inbox column (InboxRow.from_session), schema migration (ALTER TABLE for existing state.db), and all five rendering sites now produce '[mention from slug@project · reply-to <handle>]' where the handle is session-id-if-resolvable else slug@project.

## Consequences

- Reply handle is precise (exact session) when resolvable, safe (slug@project) fallback when not.
- Back-compatible: decoding an old event without the from-session tag yields from_session: None.
- Inbox PK (mention_event_id, target_session) unchanged — idempotent delivery preserved.
- Forward-looking canonical messages schema and migration plan updated to preserve author_session / return envelope so it isn't dropped during inbox→messages cutover.
- MCP channel injection path (outside src/) does NOT yet carry from_session — flagged for future treatment.

## Open Tail

- Daemon restart needed for migration to take effect; the pre-fix message sitting in codex's inbox won't retroactively gain a session id.
- MCP reply_to path needs the same from_session treatment.

## Evidence

- transcript lines 806-1567

