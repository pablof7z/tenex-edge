---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: architecture
status: active
subjects:
  - mcp-caller-identity
  - caller-anchor
  - session-registration
supersedes:
  - 2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-derivation-replaced
related_claims: []
source_lines:
  - 2029-2041
  - 2042-2044
  - 2117-2132
  - 2235-2235
  - 2384-2424
  - 2625-2629
  - 2857-2896
  - 2903-2915
captured_at: 2026-07-10T14:54:24Z
---

# Episode: MCP caller identity unified with hosted session machinery

## Prior State

MCP HTTP callers (e.g. ChatGPT) had no real tenex-edge session identity. The initial implementation bolted a one-off `mcp_derived_self` function onto `who.rs` that computed a deterministic keypair via `derive_session_keys_v2` but never persisted it — no Session row, no alias, no identity cache entry. This was a 'Potemkin identity' visible only in `who` output; every other capability (`chat_write`, `channels_join`, etc.) would have needed its own bespoke reimplementation to recognize the caller.

## Trigger

User correction at line 2042: 'your design is obviously fucked up' — publishing the identity as a channel-visible member should require no additional work because it should all be the same code. An independent Opus review (lines 2117-2127) confirmed the user's instinct: `mcp_derived_self` computes but never persists, so it's a Potemkin identity that no other handler can resolve.

## Decision

Replaced the bespoke `mcp_derived_self` path entirely. A new `mcp_session` anchor kind was added to `CallerAnchor`, and a one-time `ensure_mcp_session` call (mirroring `rpc_session_start`'s sequencing: resolve-or-mint canonical id → `mint_session_identity` → write the Session row) is invoked at MCP `initialize`, gated behind `--oauth`. After registration, the existing read-only `resolve_session_inner` finds the session through the `mcp_session` anchor exactly like any other anchor kind. The bespoke `mcp_derived_self` function, the `mcp_identity` output field, and the `mcp_provider_slug` helper were all deleted from `who.rs`.

## Consequences

- Every existing RPC handler (who, chat_write, channels_join, chat_read, channels_create, etc.) now resolves MCP callers through the same generic anchor path with zero per-tool special-casing.
- MCP caller identities are real, persisted sessions in the same table as hosted sessions, tagged harness='mcp' — no OS process is spawned behind them.
- Identity minting is gated behind --oauth, so an unauthenticated endpoint can never mint identities.
- X-Openai-Session header (sent by ChatGPT on every request) serves as the stable per-connection correlation key, filed under initialize's clientInfo.name.
- Proven end-to-end over the daemon's own Unix socket: ensure_mcp_session → who → channels_join → chat_write all resolved the same session (te-18c0f41a42d08020-0) with zero MCP-specific code in any handler.
- 941/941 tests pass, clippy clean; the bespoke mcp_derived_self tests were rewritten to exercise the real anchor path.

## Open Tail

- Not yet exercised from the real chatgpt.com UI (only proven via direct daemon socket and curl calls).
- Identity is registered but does not auto-join any channel — the caller must explicitly channels_join, same as a new human would.

## Evidence

- transcript lines 2029-2041
- transcript lines 2042-2044
- transcript lines 2117-2132
- transcript lines 2235-2235
- transcript lines 2384-2424
- transcript lines 2625-2629
- transcript lines 2857-2896
- transcript lines 2903-2915

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-unified-with-hosted.json`](transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-unified-with-hosted.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-unified-with-hosted.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-unified-with-hosted.json)
