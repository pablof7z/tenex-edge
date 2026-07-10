---
title: Tenex-Edge MCP Client Identity
slug: tenex-edge-mcp-client-identity
topic: tenex-edge
summary: "The HTTP transport tracks `Mcp-Session-Id` (format: `mcp-<nanos>-<counter>`, matching `mint_session_id()`) to correlate requests with `initialize`: at `initiali"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-10
updated: 2026-07-10
verified: 2026-07-10
compiled-from: conversation
sources:
  - session:4d65680c-ded1-47cd-a59a-4966eebe8eda
---

# Tenex-Edge MCP Client Identity

## HTTP Session Identity Correlation

The HTTP transport tracks `Mcp-Session-Id` (format: `mcp-<nanos>-<counter>`, matching `mint_session_id()`) to correlate requests with `initialize`: at `initialize` it captures `clientInfo.name` into an `HttpState.mcp_sessions` map, returns the session id via the `Mcp-Session-Id` response header (CORS-allowed), and on subsequent requests reads the incoming `Mcp-Session-Id` header to look up the stored name. <!-- [^4d656-84c25] -->

## Client Info Propagation in RPC Builders

Every RPC-building function in `tools.rs` (`call`, `who`, `chat_read`, `chat_write`, `channels_create`, `channel_mutation`, `daemon_identity`) takes `client_info_name: Option<&str>` and merges it into outgoing daemon params via a `with_client_info` helper. <!-- [^4d656-8c926] -->

## Stdio Transport Behavior

The stdio transport call site passes `None` for `client_info_name`, preserving unchanged behavior there. <!-- [^4d656-84dba] -->
