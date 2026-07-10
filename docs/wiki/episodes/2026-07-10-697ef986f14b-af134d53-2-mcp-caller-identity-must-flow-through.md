---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: architecture
status: active
subjects:
  - mcp-identity
  - caller-anchor
  - session-registration
  - derive-session-keys
supersedes: []
related_claims: []
source_lines:
  - 1165-2115
captured_at: 2026-07-10T14:19:04Z
---

# Episode: MCP caller identity must flow through CallerAnchor, not a bolted-on deriver

## Prior State

MCP callers with no agent/cwd/channel context (like ChatGPT over HTTP) had no identity — `who` failed closed with 'needs an exact live session anchor.' No mechanism existed to derive or register a Nostr keypair for these callers.

## Trigger

User demanded a testable identity and was furious that the initial transport fix was invisible. The assistant then built a one-off `mcp_derived_self` deriver bolted onto `who.rs` that computed a keypair from the provider session hint. The user immediately critiqued this as architecturally wrong: 'ALL this should be doing is creating a key based on the http header instead of a harness session-id — so your design is obviously fucked up.' The assistant agreed the one-off approach would require reimplementing every downstream capability (channel join, roster, chat) per transport.

## Decision

Feed the HTTP provider session header into the existing `CallerAnchor` / session-registration path (the same path every hosted CLI session uses via `derive_session_keys_v2`), as a new anchor kind — rather than a parallel one-off deriver in `who.rs`. All downstream capabilities (channel membership, roster visibility, chat) must work without bespoke per-transport reimplementation.

## Consequences

- The one-off `mcp_derived_self` / `mcp_provider_slug` helpers in who.rs are now historical and must be removed
- A new CallerAnchor variant for HTTP-provider-session-based callers needs to be added so resolve_session handles them through the standard path
- Session registration (channel provisioning, join events, roster visibility) will automatically work for MCP callers once they go through the standard anchor path
- The intermediate derived-identity work (commit 432dacdc) proved determinism but is architecturally superseded

## Open Tail

- CallerAnchor variant for MCP HTTP callers not yet implemented — still in progress at session end
- Full session-registration for no-anchor callers (channel join, Nostr publish) remains unbuilt

## Evidence

- transcript lines 1165-2115

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-2-mcp-caller-identity-must-flow-through.json`](transcripts/2026-07-10-697ef986f14b-af134d53-2-mcp-caller-identity-must-flow-through.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-2-mcp-caller-identity-must-flow-through.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-2-mcp-caller-identity-must-flow-through.json)
