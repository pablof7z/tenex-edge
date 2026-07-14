---
title: Mosaico MCP Server
slug: mosaico-mcp-server
topic: mosaico
summary: The MCP server debug wrapper script launches the server fully detached from the launching agent session, with a scrubbed environment (`env -i` plus only `PATH/H
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-10
updated: 2026-07-10
verified: 2026-07-10
compiled-from: conversation
sources:
  - session:45545de9-b1cd-4e9c-afdc-db305affbb87
---

# Mosaico MCP Server

## Server Launch & Process Isolation

The MCP server debug wrapper script launches the server fully detached from the launching agent session, with a scrubbed environment (`env -i` plus only `PATH/HOME/USER/TMPDIR/LANG`) so no agent identity leaks in. The launched server is reparented to `launchd` (verified `ppid=1`) so its `pty_search` walk cannot find "claude" and misattribute ChatGPT/Grok requests to the dev session. <!-- [^45545-e8f81] -->

## Access Logging & Traffic Capture

The `mcp-debug.sh` wrapper script captures all HTTP traffic — method, tool name, resource URI, `clientInfo`, and sanitized headers — per request in an access log at `.../scratchpad/mcp-debug/mcp-server.log`. <!-- [^45545-888d5] -->

## Debug Wrapper Subcommands

The `mcp-debug.sh` script exposes subcommands: `log [n]` (app access log), `status` (health + URL), `requests` (programmatic traffic pull), and `stop`. <!-- [^45545-18a64] -->

## External Endpoint Exposure

The MCP debug endpoint is exposed via ngrok with no auth required by default, and can be restarted with `--oauth --public-url` if a connector setup insists on OAuth. <!-- [^45545-6d674] -->

## `who` Tool Fallback Identity

The MCP `who` tool's auto-provision fallback identity only activates when no anchor resolves. <!-- [^45545-60117] -->
