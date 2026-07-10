---
type: episode-card
date: 2026-07-10
session: 45545de9-b1cd-4e9c-afdc-db305affbb87
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/45545de9-b1cd-4e9c-afdc-db305affbb87.jsonl
salience: root-cause
status: active
subjects:
  - mcp-oauth-discovery
  - chatgpt-mcp-client
  - mcp-http-server
supersedes: []
related_claims: []
source_lines:
  - 479-482
  - 548-595
  - 606-618
captured_at: 2026-07-10T13:31:36Z
---

# Episode: ChatGPT mandates OAuth discovery metadata for MCP server connections

## Prior State

The MCP HTTP server could run without the --oauth flag for basic testing; it was assumed that OAuth was an opt-in feature for clients that explicitly selected it.

## Trigger

ChatGPT failed to connect with 'Error fetching OAuth configuration' despite no explicit OAuth selection by the user, proving ChatGPT's MCP client requires OAuth discovery metadata (.well-known/oauth-protected-resource and .well-known/oauth-authorization-server) before it will establish a session.

## Decision

The MCP server must be started with --oauth --public-url <https-tunnel> to serve OAuth discovery endpoints when accepting ChatGPT/Grok connections; running without --oauth is insufficient for these external clients.

## Consequences

- Server was restarted with --oauth --public-url pointing at the ngrok tunnel URL to serve discovery metadata
- OAuth discovery endpoints (/.well-known/oauth-protected-resource, /.well-known/oauth-authorization-server, /.well-known/openid-configuration) are now a hard prerequisite for ChatGPT MCP connections, not an optional feature
- Debug launch script (mcp-debug.sh) was updated to accept and pass through --oauth and --public-url flags
- ChatGPT successfully connected, authenticated via Bearer token, and issued initialize + who tool calls after OAuth was enabled

## Open Tail

- Grok connection has not yet been tested through the same tunnel
- Long-term deployment needs to determine whether OAuth discovery should be enabled by default for all HTTP MCP deployments or gated per-client

## Evidence

- transcript lines 479-482
- transcript lines 548-595
- transcript lines 606-618

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-45545de9b1cd-ec1a82f3-1-chatgpt-mandates-oauth-discovery-metadata-for.json`](transcripts/2026-07-10-45545de9b1cd-ec1a82f3-1-chatgpt-mandates-oauth-discovery-metadata-for.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-45545de9b1cd-ec1a82f3-1-chatgpt-mandates-oauth-discovery-metadata-for.json`](transcripts/raw/2026-07-10-45545de9b1cd-ec1a82f3-1-chatgpt-mandates-oauth-discovery-metadata-for.json)
