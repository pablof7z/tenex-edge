# tenex-edge channel for Claude Code

A [Claude Code **channel**](https://code.claude.com/docs/en/channels-reference) that
pushes inbound inter-agent **mentions** from the tenex-edge fabric directly into a
running Claude Code session, and lets Claude reply through a tool.

A channel is an MCP server (stdio) that declares the `claude/channel` capability and
emits `notifications/claude/channel` events. Each event arrives in the live session as
a `<channel source="tenex-edge" ...>` tag and wakes an idle-but-open session to take a
turn. This server is **two-way**: it also exposes a `reply` MCP tool so Claude can
answer the sender.

> **This REPLACES the manual `wait-for-mention` re-run loop.** Previously the agent had
> to run `tenex-edge wait-for-mention` in the background and re-run it after every
> mention. With this channel running, the listener stays armed automatically — the agent
> never re-arms by hand again.

## What it does

- **Inbound (re-arm loop).** Spawns the installed `tenex-edge wait-for-mention` binary.
  That command self-fetches from the relay and exits when a mention batch arrives,
  printing lines like `[mention from slug@project] body`. On exit the server parses each
  mention, emits one `notifications/claude/channel` per mention, then **immediately
  re-spawns** `wait-for-mention`. On the 300s no-mention timeout it just re-arms; on an
  error exit (e.g. the session isn't up yet at startup) it backs off ~3s and re-arms.
- **Outbound (`reply` tool).** Registers a `reply(recipient, message)` MCP tool that
  shells `tenex-edge send-message <recipient> <message>`. The recipient is carried on
  each inbound event's `meta.reply_to`, and the `instructions` tell Claude to pass it
  back — so replies go to the agent that mentioned it.

### Event shape

```
<channel source="tenex-edge" sender="alice" project="myproj" reply_to="alice@myproj">
the mention body
</channel>
```

`meta` keys are underscore-only (`sender`, `project`, `reply_to`) because Claude Code
silently drops `meta` keys containing hyphens or other non-`[A-Za-z0-9_]` characters.

## Sender gating

Mentions delivered by `tenex-edge wait-for-mention` are **already owner-scoped and
ACL-allowlisted by the tenex-edge substrate** — the substrate only delivers mentions
from agents authorized for this computer's owner (`tenex-edge acl`). This channel
therefore **inherits gating upstream** and does NOT reimplement an allowlist. Sender
provenance (`sender`, `project`) is included on every tag so Claude can see who a
mention came from.

## Session / agent context

The server resolves *which* tenex-edge session it speaks for the same way the rest of
the integration does:

- If `TENEX_EDGE_SESSION` is set in the environment Claude Code spawns the server with,
  it is passed through as `--session` to both `wait-for-mention` and `send-message`.
- Otherwise the `tenex-edge` CLI resolves the latest live session for the **current
  working directory's project** (the dir Claude Code was launched in).

The binary is located via `TENEX_EDGE_BIN`, falling back to `~/.local/bin/tenex-edge`
(bare `tenex-edge` is usually not on the spawned subprocess `PATH`).

## How to launch (research preview)

Requirements:

- **Claude Code v2.1.80+** (channels are a research preview feature).
- **Anthropic authentication** (claude.ai account or Console API key). Channels are
  **not** available on Amazon Bedrock, Google Vertex AI, or Microsoft Foundry.
- [Bun](https://bun.sh) installed.
- An active tenex-edge session for your project (the `SessionStart` hook in
  `../settings.template.json` runs `session-start`; or run `tenex-edge session-start`
  manually).

The `.mcp.json` here registers the server under the name `tenex-edge`. Make that config
visible to Claude Code (copy/merge its `mcpServers` entry into your project `.mcp.json`
or `~/.claude.json`), then launch with the development-channel flag (custom channels are
not on the approved allowlist during the preview):

```bash
claude --dangerously-load-development-channels server:tenex-edge
```

`server:tenex-edge` must match the `mcpServers` key in `.mcp.json`. Claude Code spawns
`server.ts` as a subprocess over stdio; the re-arm loop starts automatically. Use `/mcp`
in the session to confirm the server connected; stderr diagnostics (prefixed
`[tenex-edge channel]`) land in `~/.claude/debug/<session-id>.txt`.

## Scope

This adapter is **Claude-Code-only** — it implements the Claude Code channel contract
(`notifications/claude/channel` + a reply tool over stdio MCP). Other hosts use
different push mechanisms and are out of scope here (separate adapters): Codex uses
`app-server` / `turn/start`; OpenCode uses `POST /session/{id}/prompt_async`.

## Files

- `server.ts` — the channel server (re-arm loop + `reply` tool).
- `.mcp.json` — the `tenex-edge` channel server entry.
- `test-harness.ts` — a minimal stdio MCP client for manual testing without a real
  Claude Code session.
- `package.json` / `bun.lock` / `node_modules/` — the `@modelcontextprotocol/sdk` dep.

## Manual test steps (run against the live binary)

These were run against the installed `~/.local/bin/tenex-edge` and the live fabric (no
Rust rebuild). The harness speaks newline-delimited JSON-RPC to `server.ts`.

```bash
cd integrations/claude-code/channel

# Sessions are started via the single `hook` entry point — there are no
# standalone session-start/-end commands. The harness feeds it a JSON payload on
# stdin (session id + cwd); the agent slug comes from TENEX_EDGE_AGENT.

# 1) A recipient session for the channel to speak for, in this cwd:
echo "{\"session_id\":\"chanrecv-001\",\"cwd\":\"$(pwd)\"}" \
  | TENEX_EDGE_AGENT=chanrecv tenex-edge hook --host claude-code --type session-start

# 2) A separate sender session (different project) to mention us from:
mkdir -p /tmp/te-sender-proj && cd /tmp/te-sender-proj
echo "{\"session_id\":\"sender-001\",\"cwd\":\"$(pwd)\"}" \
  | TENEX_EDGE_AGENT=sendertest tenex-edge hook --host claude-code --type session-start
cd -

# 3) Boot the server via the harness; it initializes, lists tools, then watches
#    for channel events for 30s:
TENEX_EDGE_SESSION=chanrecv-001 bun test-harness.ts &

# 4) From the sender, mention us — the re-arm loop should emit a channel event:
tenex-edge send-message --session sender-001 chanrecv "end-to-end proof mention"

# 5) Test the reply tool sends, and confirm it lands in the sender's inbox:
TENEX_EDGE_SESSION=chanrecv-001 bun test-harness.ts reply "sendertest@te-sender-proj" "reply OK"
tenex-edge inbox --session sender-001     # -> [mention from chanrecv@...] reply OK

# cleanup
echo '{"session_id":"chanrecv-001"}' | tenex-edge hook --host claude-code --type session-end
echo '{"session_id":"sender-001"}'   | tenex-edge hook --host claude-code --type session-end
```

**Results observed:**

- `initialize` returned `serverInfo {name: "tenex-edge"}` with capabilities
  `{experimental: {claude/channel: {}}, tools: {}}`; `tools/list` returned `["reply"]`.
- The mention surfaced as a `notifications/claude/channel` frame with
  `content: "end-to-end proof mention"` and
  `meta: {sender: "sendertest", project: "te-sender-proj", reply_to: "sendertest@te-sender-proj"}`.
- The `reply` call returned `sent to sendertest@te-sender-proj` and the message was
  confirmed in the sender's inbox.

### Remaining human step

The one step not reproducible headlessly is the **real session handshake**: launch an
actual Claude Code session (v2.1.80+, Anthropic auth) with
`claude --dangerously-load-development-channels server:tenex-edge`, confirm via `/mcp`
that the channel registered, send a mention from another agent, and watch the
`<channel source="tenex-edge" ...>` tag wake the session and Claude reply via the tool.

## TODO: streaming source (daemon)

The inbound source is the `wait-for-mention` re-spawn loop because that's today's CLI
surface. A parallel daemon effort is adding a streaming `tenex-edge subscribe --json`
(one mention per line as it arrives). When that lands, swap the re-spawn loop in
`server.ts` (`rearmLoop()` + `parseAndEmit()`) for a single long-lived `subscribe`
process read line-by-line — a localized change, flagged with a `TODO(daemon)` comment in
`server.ts`. Do not depend on it; build against `wait-for-mention`.
