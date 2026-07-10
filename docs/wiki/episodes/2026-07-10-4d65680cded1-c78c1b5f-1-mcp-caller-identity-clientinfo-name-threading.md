---
type: episode-card
date: 2026-07-10
session: 4d65680c-ded1-47cd-a59a-4966eebe8eda
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d65680c-ded1-47cd-a59a-4966eebe8eda.jsonl
salience: architecture
status: active
subjects:
  - mcp-caller-identity
  - client-info-name
  - mcp-session-id
  - resolve-session-inner
supersedes:
  - 2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-auto-provisioning-from
related_claims: []
source_lines:
  - 1-49
  - 50-65
  - 259-264
  - 366-397
  - 448-457
captured_at: 2026-07-10T13:34:52Z
---

# Episode: MCP caller identity: clientInfo.name threading toward auto-provisioning

## Prior State

When the MCP server started without agent context (no PTY session, no harness, no watch-pid), it silently degraded to caller_rec = None — an anonymous, session-less fabric snapshot with no self-row and no (you) marker. clientInfo.name from the MCP initialize handshake was captured in access logs but discarded, never threaded through to session resolution. No MCP-Session-Id tracking existed for the HTTP transport, making per-connection correlation impossible.

## Trigger

User directive: 'with no agent context it should create an agent context for that session, ideally showing it as chatgpt/echo123 for chatgpt or grok/echo123 for grok.' User further clarified the fix belongs at the identity-setting layer (resolve_session_inner), not scoped narrowly to rpc_who: 'not rpc_who; that's too narrow — it should be set to whatever sets the agent identity, not just who.'

## Decision

Thread clientInfo.name from the MCP initialize handshake through to daemon RPC calls as a new client_info_name field. For HTTP transport (the one ChatGPT uses), this required adding Mcp-Session-Id issuance and in-memory tracking: on initialize, mint a session id, capture clientInfo.name, return the id via response header; on later requests, read the header to recover the stored name. Every RPC-building function in tools.rs (who, chat_read, chat_write, channels_create, channel_mutation, daemon_identity) now accepts and forwards client_info_name. stdio transport passes None (deferred). The designated choke point for the eventual auto-provisioning is resolve_session_inner, not any single RPC handler — so all 11 caller sites get JIT identity for free once the daemon side is built.

## Consequences

- MCP HTTP transport now issues and tracks Mcp-Session-Id headers, enabling per-connection correlation where none existed before.
- client_info_name is on the wire for every daemon RPC call from the HTTP transport, but the daemon side (CallerAnchor/resolve_session_inner) does not yet consume it — auto-provisioning logic is unbuilt.
- who will eventually transform from a pure read into a side-effecting call (creates a session record on first contact) once the daemon-side change lands.
- clientInfo.name is identity-by-assertion (self-reported by the client), suitable for display/awareness but not security-sensitive decisions.
- The mcp_sessions in-memory map has no eviction policy — long-lived tunnels accumulate one entry per distinct MCP client connection.
- stdio transport is unchanged (passes None), so non-HTTP MCP callers still get anonymous degradation.

## Open Tail

- Daemon-side auto-provisioning: resolve_session_inner needs to synthesize an agent_slug (<clientinfo-name>/<random-suffix>) and call Store::register_session when no anchor resolves and client_info_name is present.
- Decision needed: whether Strict-scope callers (chat_write, turn_start, invite) should auto-provision on first touch, or whether JIT creation should stay narrower to avoid surprising the explicit session-start contract.
- stdio transport still needs clientInfo.name threading (deferred this session).
- End-to-end testing against a real ChatGPT/Grok connection to confirm initialize round-trips Mcp-Session-Id correctly.

## Evidence

- transcript lines 1-49
- transcript lines 50-65
- transcript lines 259-264
- transcript lines 366-397
- transcript lines 448-457

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-clientinfo-name-threading.json`](transcripts/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-clientinfo-name-threading.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-clientinfo-name-threading.json`](transcripts/raw/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-clientinfo-name-threading.json)
