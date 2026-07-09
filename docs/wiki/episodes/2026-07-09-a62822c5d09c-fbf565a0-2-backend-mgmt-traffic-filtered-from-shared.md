---
type: episode-card
date: 2026-07-09
session: a62822c5-d09c-4a83-9251-a3856d276ac4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a62822c5-d09c-4a83-9251-a3856d276ac4.jsonl
salience: architecture
status: active
subjects:
  - mgmt-commands
  - channel-read-visibility
  - backend-traffic-filtering
supersedes:
  - 2026-07-09-a62822c5d09c-fbf565a0-2-mgmt-backend-exchanges-filtered-from-agent
related_claims: []
source_lines:
  - 29-31
  - 78-87
  - 1109-1110
captured_at: 2026-07-09T20:52:45Z
---

# Episode: Backend/mgmt traffic filtered from shared channel reads

## Prior State

Management commands (`list agents`, `list sessions`, etc.) were published as genuine kind:9 nostr chat events to the relay — same channel, same visibility as any agent message. The only targeting was a p-tag/recipient row for @mention rendering, which is not an access-control or visibility field. No private/ephemeral/non-broadcast concept existed in the system. Other agents reading the channel would see mgmt round-trips.

## Trigger

User observed a mgmt-agent conversation (`list agents` → `mgmt ok: 14 agent(s)`) leaking into `tenex-edge channel read` output, which is supposed to be invisible to agents (line 29).

## Decision

`handle_chat_read` now filters out backend-authored or backend-p-tagged rows from both the initial batch and the `--live` tail, reusing the existing `is_backend_traffic` check that already protected the hook/awareness snapshot path (`fabric_context::messages::is_backend_traffic`), bumped to `pub(crate)`.

## Consequences

- Mgmt round-trips no longer appear when another agent reads the channel via `chat_read`
- The existing backend-traffic invariant from the hook/awareness path is now enforced on the CLI read path too, establishing a uniform visibility filter across all read surfaces
- Mgmt commands are still published to the relay (publish model unchanged) but are now suppressed from agent-facing reads
- Regression tests added for filtering by both author-pubkey and p-tagged recipient

## Open Tail

*(none)*

## Evidence

- transcript lines 29-31
- transcript lines 78-87
- transcript lines 1109-1110

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-09-a62822c5d09c-fbf565a0-2-backend-mgmt-traffic-filtered-from-shared.json`](transcripts/2026-07-09-a62822c5d09c-fbf565a0-2-backend-mgmt-traffic-filtered-from-shared.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-2-backend-mgmt-traffic-filtered-from-shared.json`](transcripts/raw/2026-07-09-a62822c5d09c-fbf565a0-2-backend-mgmt-traffic-filtered-from-shared.json)
