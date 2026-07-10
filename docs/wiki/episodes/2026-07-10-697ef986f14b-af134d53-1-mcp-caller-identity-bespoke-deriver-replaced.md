---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: reversal
status: superseded
subjects:
  - mcp-caller-identity
  - caller-anchor
  - session-registration
supersedes:
  - 2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-who-rs
related_claims: []
source_lines:
  - 2042-2044
  - 2117-2131
  - 2235-2235
  - 2384-2488
  - 2625-2629
  - 2857-2896
  - 2903-2915
captured_at: 2026-07-10T14:42:51Z
---

# Episode: MCP caller identity: bespoke deriver replaced by real session registration

## Prior State

MCP HTTP callers (e.g. ChatGPT) had no real tenex-edge session identity. A bespoke `mcp_derived_self` function was bolted onto `who.rs` that computed a deterministic keypair via `derive_session_keys_v2` but never persisted it — no session row, no identity cache entry, no alias. It was a 'Potemkin identity': `who` could display a pubkey, but `chat_write`, `channels_join`, and every other tool would each have needed their own bespoke reimplementation to recognize the same caller.

## Trigger

User explicitly rejected the design (line 2042): 'your design is obviously fucked up — ALL this should be doing is creating a key based on the http header instead of a harness session-id.' An independent Opus review (lines 2117-2131) confirmed the root cause: the bespoke path 'computes but never persists,' so it doesn't integrate with the session/anchor machinery that every other capability depends on.

## Decision

Deleted the entire `mcp_derived_self` / `mcp_identity` special-casing from `who.rs`. Added a new `mcp_session` anchor kind to `CallerAnchor`, resolved read-only by the existing `resolve_session_inner`. A new `ensure_mcp_session` function (called once at MCP `initialize`, gated behind `--oauth`) mirrors `rpc_session_start`'s sequencing: resolve-or-mint canonical session id → `mint_session_identity` → write the session row — but without spawning an OS process. The `X-Openai-Session` HTTP header (which ChatGPT sends on every request) is the stable correlation key, replacing a custom session header that ChatGPT was ignoring.

## Consequences

- Every pre-existing RPC handler (who, chat_write, channels_join, channels_create, etc.) now resolves MCP callers through the same generic `CallerAnchor` path with zero per-tool special-casing — proven live over the daemon's Unix socket: ensure_mcp_session → who → channels_join → chat_write all resolved the same session (te-18c0f41a42d08020-0, pubkey cae33c31...).
- Unauthenticated MCP endpoints can never mint identities because `ensure_mcp_session` is gated behind `--oauth` at the HTTP transport layer.
- MCP callers get a real, persisted, channel-visible Nostr identity — channels_join publishes a real join event, chat_write publishes signed events — because the session row carries the minted ordinal pubkey exactly like a hosted session.
- 941/941 tests pass; bespoke test for `mcp_derived_self` replaced by tests exercising the real anchor resolution (fails closed for un-ensured hints, resolves through normal path once ensured).

## Open Tail

- Not yet exercised from the real chatgpt.com UI (only verified via raw Unix socket and local HTTP).
- Isolated sandbox daemon used for testing has empty whitelistedPubkeys, causing OAuth login failures for real ChatGPT connectors — needs the operator's pubkey added to the sandbox config before live UI testing.

## Evidence

- transcript lines 2042-2044
- transcript lines 2117-2131
- transcript lines 2235-2235
- transcript lines 2384-2488
- transcript lines 2625-2629
- transcript lines 2857-2896
- transcript lines 2903-2915

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-deriver-replaced.json`](transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-deriver-replaced.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-deriver-replaced.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-deriver-replaced.json)
