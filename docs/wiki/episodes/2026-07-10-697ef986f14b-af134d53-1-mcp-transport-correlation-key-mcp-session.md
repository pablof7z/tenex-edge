---
type: episode-card
date: 2026-07-10
session: 697ef986-f14b-4fcc-ba7d-b9f0d1b065ad
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/697ef986-f14b-4fcc-ba7d-b9f0d1b065ad.jsonl
salience: product
status: active
subjects:
  - mcp-transport
  - session-correlation
  - client-identity
supersedes:
  - 2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-clientinfo-name-threading
related_claims: []
source_lines:
  - 925-1151
captured_at: 2026-07-10T14:19:04Z
---

# Episode: MCP transport correlation key: Mcp-Session-Id replaced by X-Openai-Session

## Prior State

MCP HTTP transport minted a custom `Mcp-Session-Id` header at `initialize` and expected clients to echo it back on subsequent `tools/call` requests to recover which client's `initialize` the request belonged to. The clientInfo.name (e.g. 'ChatGPT') was stored keyed by this minted session id.

## Trigger

Real captured ChatGPT traffic analysis revealed that ChatGPT silently ignores the `Mcp-Session-Id` header — it never echoes it back on `tools/call`. Every `tools/call` from ChatGPT arrived with no correlation key, so `client_info_name` was always `None` for all subsequent requests.

## Decision

Use the provider's own `X-Openai-Session` header (which ChatGPT sends unprompted on every request) as the primary `provider_session_key` for correlation, keeping `Mcp-Session-Id` as a spec-compliant fallback for clients that do echo it. The session map is now keyed by whichever stable per-request header the client actually sends.

## Consequences

- Identity now resolves correctly for real ChatGPT traffic without requiring client cooperation with a custom header
- The McpSessions map is unbounded — entries are never evicted, so a long-lived tunnel accumulates one row per distinct MCP client session
- 4 unit tests added proving resolution logic against the exact captured ChatGPT traffic pattern (provider header only, no Mcp-Session-Id echo)
- CORS Access-Control-Allow-Headers and Expose-Headers still reference mcp-session-id for spec compliance

## Open Tail

- Session map has no eviction policy — long-lived tunnels accumulate unbounded entries

## Evidence

- transcript lines 925-1151

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-transport-correlation-key-mcp-session.json`](transcripts/2026-07-10-697ef986f14b-af134d53-1-mcp-transport-correlation-key-mcp-session.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-transport-correlation-key-mcp-session.json`](transcripts/raw/2026-07-10-697ef986f14b-af134d53-1-mcp-transport-correlation-key-mcp-session.json)
