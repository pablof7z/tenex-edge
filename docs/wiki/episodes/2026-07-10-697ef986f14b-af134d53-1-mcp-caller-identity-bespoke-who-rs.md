---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: reversal
status: active
subjects:
  - mcp-caller-identity
  - caller-anchor
  - session-registration
supersedes:
  - 2026-07-10-697ef986f14b-af134d53-2-mcp-caller-identity-must-flow-through
related_claims: []
source_lines:
  - 2042-2044
  - 2117-2132
  - 2235-2235
  - 2625-2629
  - 2857-2896
  - 2903-2915
captured_at: 2026-07-10T14:35:58Z
---

# Episode: MCP caller identity: bespoke who.rs deriver replaced by real session registration via shared anchor path

## Prior State

MCP HTTP callers (e.g. ChatGPT) had a one-off keypair derivation bolted onto who.rs (`mcp_derived_self`) that computed a pubkey via `derive_session_keys_v2` but never persisted it — no session row, no alias, no identity table entry. The derived identity was a 'Potemkin identity' visible only in who output, meaning every other capability (chat_write, channels_join, etc.) would each need their own bespoke reimplementation to know who the caller is.

## Trigger

User correction at line 2042: 'you are clearly re-implementing things — ALL this should be doing is creating a key based on the http header instead of a harness session-id — so your design is obviously fucked up.' Confirmed by independent Opus review (lines 2117-2131) which verified the principle was right but the mechanism was wrong: the computed identity existed nowhere, so no downstream handler could reuse it.

## Decision

Replaced the bespoke who.rs deriver with `ensure_mcp_session`, called once at MCP `initialize` (gated behind --oauth), which mirrors `rpc_session_start`'s sequencing: resolve-or-mint canonical id → `mint_session_identity` → write the session row. A new `mcp_session` CallerAnchor kind was added to the existing `resolve_session_inner` read path, resolved identically to `pty_session`/`harness_session`. The X-Openai-Session HTTP header (which ChatGPT sends on every request) is the stable per-connection key, replacing a prior custom session header that ChatGPT was silently ignoring. All bespoke MCP special-casing in who.rs was deleted.

## Consequences

- All pre-existing generic RPC handlers (who, chat_write, channels_join, chat_read, channels_create) now work for MCP callers with zero per-tool special-casing — proven live over the daemon's Unix socket with real channels_join and chat_write calls resolving the same registered session.
- MCP caller sessions are gated behind --oauth so an unauthenticated endpoint can never mint identities.
- MCP sessions are tagged harness="mcp" but have no backing OS process (unlike hosted CLI sessions), so they are registered without spawning a process.
- The caller has an identity but does not auto-join any channel — explicit channels_join is still required, same as a new human.
- 941/941 tests pass; the bespoke mcp_derived_self, mcp_provider_slug, and mcp_identity output field were removed entirely.
- Publishing the identity as a channel-visible Nostr member (join event) remains a separate, not-yet-implemented step.

## Open Tail

- MCP caller identity as a channel-visible member (Nostr join event) is still not implemented — the session exists but is not yet published to the relay network.
- Not yet exercised end-to-end from the real chatgpt.com UI (verified via raw daemon socket, not the ChatGPT connector).

## Evidence

- transcript lines 2042-2044
- transcript lines 2117-2132
- transcript lines 2235-2235
- transcript lines 2625-2629
- transcript lines 2857-2896
- transcript lines 2903-2915

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-who-rs.json`](transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-who-rs.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-who-rs.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-caller-identity-bespoke-who-rs.json)
