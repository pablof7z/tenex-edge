---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: architecture
status: superseded
subjects:
  - mcp-caller-identity
  - session-anchor
  - calleranchor
supersedes:
  - 2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-deriver-replaced
related_claims: []
source_lines:
  - 2029-2044
  - 2117-2132
  - 2235-2235
  - 2625-2629
  - 2896-2915
  - 2973-2984
captured_at: 2026-07-10T14:47:25Z
---

# Episode: MCP caller identity: bespoke derivation replaced by real session registration

## Prior State

MCP HTTP callers (e.g. ChatGPT) had no real tenex-edge session identity. The initial implementation bolted a one-off `mcp_derived_self` function onto `who.rs` that computed a deterministic keypair via `derive_session_keys_v2` but never persisted it — no `Session` row, no `identities` entry, no alias. This was a 'Potemkin identity': `who` could report a pubkey, but `chat_write`, `channels_join`, and every other RPC handler would each have needed their own bespoke reimplementation to use it.

## Trigger

User correction (line 2042): 'none of the stuff like publishing the identity as a real, channel-visible member should require ANY other work — this should ALL be the same code — so you are clearly re-implementing things — so your design is obviously fucked up.' An independent Opus review (line 2117) confirmed the diagnosis: the identity was computed but never persisted, so it existed nowhere in the session/anchor tables.

## Decision

Replaced the bespoke `mcp_derived_self` path with a new `mcp_session` `CallerAnchor` kind. MCP callers are now registered exactly once at `initialize` (gated behind `--oauth`) via `ensure_mcp_session`, which mirrors `rpc_session_start`'s sequencing (resolve-or-mint canonical session id → `mint_session_identity` → write the session row) but without spawning an OS process. After registration, the existing `resolve_session_inner` finds the session read-only through the `mcp_session` anchor — identical to how `pty_session` and `harness_session` anchors work. All existing RPC handlers (`who`, `chat_write`, `channels_join`, etc.) resolve the caller through this shared path with zero per-tool MCP-specific code.

## Consequences

- MCP callers now possess real, persisted sessions with deterministic Nostr keypairs keyed off `X-Openai-Session`, same derivation scheme as hosted sessions.
- Every existing tool handler works for MCP callers automatically — `who`, `channels_join`, `chat_write` all resolved the same session (`te-18c0f41a42d08020-0`) in live socket-level testing with zero handler modifications.
- Identity minting is gated behind `--oauth`; an unauthenticated endpoint can never mint identities.
- The `mcp_derived_self` helper, `mcp_provider_slug` helper, and the special `mcp_identity` output field in `who.rs` were deleted entirely.
- MCP callers must explicitly `channels_join` to become channel-visible — registration gives identity, not channel membership, same as a new human.
- 941/941 tests pass including new tests verifying: fails-closed for un-ensured hints, resolves through the normal anchor path once ensured, stable/collision-free derivation.

## Open Tail

- Not yet exercised from the real chatgpt.com UI (only proven over the daemon's Unix socket).
- Disk-full condition on the machine prevented final redeployment verification from chatgpt.com.

## Evidence

- transcript lines 2029-2044
- transcript lines 2117-2132
- transcript lines 2235-2235
- transcript lines 2625-2629
- transcript lines 2896-2915
- transcript lines 2973-2984

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-derivation-replaced.json`](transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-derivation-replaced.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-derivation-replaced.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-derivation-replaced.json)
