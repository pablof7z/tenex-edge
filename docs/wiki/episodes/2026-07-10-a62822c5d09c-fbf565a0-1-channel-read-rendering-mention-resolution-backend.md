---
type: episode-card
date: 2026-07-10
session: a62822c5-d09c-4a83-9251-a3856d276ac4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a62822c5-d09c-4a83-9251-a3856d276ac4.jsonl
salience: product
status: active
subjects:
  - channel-read
  - mention-resolution
  - backend-traffic-filtering
  - whitelisted-operator-rendering
supersedes:
  - 2026-07-09-a62822c5d09c-fbf565a0-1-channel-read-mention-resolution-wire-existing
  - 2026-07-09-a62822c5d09c-fbf565a0-2-mgmt-backend-exchanges-filtered-from-agent
  - 2026-07-09-a62822c5d09c-fbf565a0-3-whitelisted-human-operators-render-as-bare
related_claims: []
source_lines:
  - 28-31
  - 52-76
  - 78-87
  - 1107-1115
captured_at: 2026-07-10T10:32:45Z
---

# Episode: Channel read rendering: mention resolution, backend-traffic filtering, whitelisted-operator label

## Prior State

`channel read` (CLI `chat` subcommand) printed message bodies verbatim — `nostr:npub1…` mentions were never resolved to `@name` because the existing `rewrite_body_mentions` resolver was invoked on the fabric-context/turn-context paths but not on the `chat_read_tail.rs` server-side path. Management/backend exchanges (e.g. `list agents`) were published as normal kind:9 chat events to the shared NIP-29 channel and appeared when any agent read the channel — the `is_backend_pubkey` check that protected the hook/awareness snapshot was not applied to CLI channel reads. Whitelisted human operators (no session/host) rendered as `<Pablo@?>` because `chat_row_refs` fell through to a `?` placeholder for missing host.

## Trigger

User ran `tenex-edge channel read` and reported two bugs: (1) nostr npub names did not resolve in the formatting, and (2) a mgmt-agent conversation (the `list agents` round-trip) was visible in the shared channel history, which is supposed to be invisible to agents.

## Decision

Three fixes committed together (66f40fee / PR #327): (1) `chat_row_to_json`/`chat_log_row_to_json` in `chat_read_tail.rs` now calls `profile::rewrite_body_mentions` (with pre-warm via `body_mention_pubkeys`) on message bodies before rendering — the same resolver already used by `fabric_context/messages.rs`. (2) `handle_chat_read` now filters out backend-authored and backend-p-tagged rows in both the initial batch and `--live` tail, reusing `fabric_context::messages::is_backend_traffic` (bumped to `pub(crate)`). (3) `chat_row_refs` short-circuits to a bare empty host for whitelisted pubkeys, and `render_chat_read_row` in `cli/messaging.rs` prints `<@Pablo>` instead of `<Pablo@?>` when host is empty.

## Consequences

- The existing `rewrite_body_mentions` is now the single source of truth for mention resolution across all render paths — no path returns raw npub strings.
- Backend/mgmt traffic filtering is now applied consistently to both the hook/awareness snapshot and the CLI channel read, closing the visibility gap.
- Whitelisted operators render as bare `<@name>` matching the convention already used for terminal-injected mentions in `injection.rs`.
- Regression tests added for all three: mention rewrite, backend-row filtering by author and by p-tagged recipient, whitelisted-host bare rendering.

## Open Tail

*(none)*

## Evidence

- transcript lines 28-31
- transcript lines 52-76
- transcript lines 78-87
- transcript lines 1107-1115

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-a62822c5d09c-fbf565a0-1-channel-read-rendering-mention-resolution-backend.json`](transcripts/2026-07-10-a62822c5d09c-fbf565a0-1-channel-read-rendering-mention-resolution-backend.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-a62822c5d09c-fbf565a0-1-channel-read-rendering-mention-resolution-backend.json`](transcripts/raw/2026-07-10-a62822c5d09c-fbf565a0-1-channel-read-rendering-mention-resolution-backend.json)
