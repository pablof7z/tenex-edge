---
type: episode-card
date: 2026-07-10
session: 4d65680c-ded1-47cd-a59a-4966eebe8eda
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/4d65680c-ded1-47cd-a59a-4966eebe8eda.jsonl
salience: architecture
status: superseded
subjects:
  - mcp-identity-resolution
  - client-info-threading
  - http-session-tracking
supersedes: []
related_claims: []
source_lines:
  - 50-61
  - 65-81
  - 83-83
  - 259-264
  - 366-398
  - 448-457
captured_at: 2026-07-10T13:27:38Z
---

# Episode: MCP caller identity auto-provisioning from clientInfo.name

## Prior State

When the MCP server was started without agent context (no PTY session, no harness id, no watch_pid), the `who` tool degraded gracefully to an anonymous, session-less fabric snapshot with no self-row and no `(you)` marker. The MCP `initialize` handshake already received `clientInfo.name` from connecting clients and logged it in `access_log.rs`, but the value was discarded — never threaded through to daemon-side session resolution or identity synthesis. No `Mcp-Session-Id` tracking existed in the HTTP transport, making per-connection state correlation impossible.

## Trigger

User directive (line 50): 'with no agent context it should create an agent context for that session, ideally showing it as chatgpt/echo123 for chatgpt or grok/echo123 for grok.' User then narrowed scope (line 65): 'not rpc_who; that's too narrow — it should be set to whatever sets the agent identity, not just who,' pointing to `resolve_session_inner` as the correct choke point. User directed implementation order (line 83): 'thread the clientInfo.name through to the daemon RPC calls,' and chose HTTP-only for the first pass (line 264).

## Decision

Adopt `clientInfo.name` (self-reported by the MCP client at `initialize` time) as the identity source for auto-provisioning sessions when no agent context exists. The correct insertion point is `resolve_session_inner` (the shared resolution function used by all 11 RPC handlers), not `rpc_who` narrowly. For this session: implemented the transport-layer plumbing only — added `Mcp-Session-Id` issuance/tracking to the HTTP transport (mints `mcp-<nanos>-<counter>` id at `initialize`, stores `clientInfo.name` in an in-memory `mcp_sessions` map, returns id via response header, reads it back on subsequent requests), and threaded `client_info_name: Option<&str>` through every RPC-building function in `tools.rs` via a new `with_client_info` merge helper. stdio transport passes `None` (deferred).

## Consequences

- `clientInfo.name` is now on the wire as `"client_info_name"` in daemon RPC params for HTTP transport, but nothing on the daemon side consumes it yet — `CallerAnchor`/`resolve_session_inner` still ignore it
- The auto-provisioning logic (synthesize `agent_slug` like `chatgpt/echo123` and call `Store::register_session` when `resolve_session_inner` hits its failure branch) is designed but not yet implemented
- The `mcp_sessions` map in `HttpState` has no eviction policy — long-lived tunnels accumulate one entry per distinct MCP client connection
- Identity is by self-assertion (client-reported `clientInfo.name`), not verified — acceptable for display/awareness, not security-sensitive operations
- stdio transport identity threading is deferred — only HTTP (the transport ChatGPT uses) is covered in this session
- In-progress edits were stashed when another developer agent checked out `master` in the shared working directory, requiring coordination before further work

## Open Tail

- Daemon-side consumption: `CallerAnchor` and `resolve_session_inner` need to read `client_info_name` and auto-provision a session (synthesize slug, call `register_session`) when no anchor resolves
- Decide whether Strict-scope callers (chat_write, turn_start, invite) should auto-provision on first touch or if JIT creation should stay narrower
- Add eviction/limit to `mcp_sessions` map
- stdio transport threading of `client_info_name` (deferred this session)
- Recover stashed edits (`stash@{0}`) and coordinate with the other developer agent working on OAuth/server in the same directory

## Evidence

- transcript lines 50-61
- transcript lines 65-81
- transcript lines 83-83
- transcript lines 259-264
- transcript lines 366-398
- transcript lines 448-457

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-auto-provisioning-from.json`](transcripts/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-auto-provisioning-from.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-auto-provisioning-from.json`](transcripts/raw/2026-07-10-4d65680cded1-c78c1b5f-1-mcp-caller-identity-auto-provisioning-from.json)
