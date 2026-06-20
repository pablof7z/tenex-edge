---
title: TENEX CLI Interface
slug: tenex-cli-interface
topic: architecture
summary: The message sending and reading commands are unified under the `inbox` command (`inbox send`, `inbox reply`, `inbox`), replacing the previous `send-message` com
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-16
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:9ac666e5-b468-4af2-be5e-83e5c8f2d1d2
  - session:435ec383-d607-459b-a712-a00ed4decaa7
  - session:cd74a605-9f83-4e21-a885-4d900e88ce07
  - session:622711fa-5176-4580-b311-d66446c2924b
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-14T13-19-49-019ec5a5-1119-76f0-a7e3-36bc985a31bd
  - session:rollout-2026-06-16T14-11-38-019ed021-38a8-7472-bc5d-dc019a072086
  - session:ses_15544a0c8ffeTRok1tpY00hCS9
  - session:ses_1307cfa82ffezNqP0fk6nYNJvs
---

# TENEX CLI Interface

## Interface Design

The message sending and reading commands are unified under the `inbox` command (`inbox send`, `inbox reply`, `inbox`), replacing the previous `send-message` command with no backward compatibility. The `inbox`, `who`, and `wait-for-mention` subcommands are user-facing CLI verbs for manual or agent-driven actions. The `session-start`, `session-end`, `turn-start`, `turn-check`, and `turn-end` subcommands must not exist as user-facing CLI verbs; they are internal functions called exclusively by the `hook` subcommand. The `tenex-edge inbox` command is no longer needed as a default spawn prompt since all local sessions are resumable. (Previously: the `send-message` CLI used `--recipient` and `--message` flags, not positional arguments.) The replaced `send-message` CLI accepted recipient and message as positional arguments, with `--session` remaining an optional flag; when the message positional argument was omitted, it read the message body from stdin, falling back only when stdin was not a terminal to avoid hanging; `-` as the message argument forced reading from stdin for explicit heredoc/pipe use, and when reading from stdin, one trailing newline was stripped to accommodate standard heredoc/pipe behavior.

The CLI module is split from the monolith into compiled implementation modules (messaging, who, turn, admin, hooks), with the monolith serving as the command enum/dispatcher.

<!-- citations: [^rollo-49] [^95659-2] [^9ac66-1] [^cd74a-1] [^62271-1] [^rollo-14] [^ses_1-2] -->
## Command Details

The command to reply to an inbox message is `tenex-edge inbox reply --id <message_id> "<reply text>"`. The `inbox reply --id` command looks up the row by event-id prefix and derives the e-tag and p-tag automatically. The reply command defaults the subject to 'Re: <original subject>' if no subject is provided. `inbox send` takes two mutually-exclusive addressing flags (enforced by a required clap `ArgGroup` — exactly one is required, and there is **no** `--to`): `--to-session <id|codename>` messages an existing session (e.g. `bravo4217`), and `--to-new-session <agent>` spawns a fresh session of that agent (project from `--project` or the cwd) and delivers the message to it. To reply upstream to a received message, use `inbox reply` instead. The codename (NATO phonetic word + 4-digit number) is a display/addressing convenience whose space is 26×10000 = 260000, so it is not collision-free at scale and is never used as identity. The underlying `send_message` RPC still resolves a raw pubkey/`slug@project` recipient for untargeted delivery (the path remote inbound mentions exercise), but the CLI never emits those forms. The `tenex-edge inbox send` command fails before publish when the recipient cannot be resolved, reporting 'no presence/profile seen yet'. The `inbox` command must remain available because it serves the opencode injection path and manual message-checking use cases; it is not redundant despite `turn-start` draining on the next prompt. The `inbox` help string must state it serves the opencode injection path and manual use, noting that Claude Code and Codex drain via the hook path (rather than claiming it is 'used by the injection hook'). The MCP `reply` tool continues to work unchanged by calling the `send_message` RPC, now with an optional subject parameter. The CLI `inbox send` handler forwards the recipient, subject, message, session, environment session/agent variables, current working directory, and thread ID to the daemon via `daemon_call_async("send_message", ...)`.

<!-- citations: [^9ac66-2] [^435ec-1] [^cd74a-2] [^rollo-74] [^ses_1-30] -->
## Testing

Test helpers (`run_cli_stdin`) must strip ambient `TENEX_EDGE_AGENT` and `TENEX_EDGE_SESSION` environment variables to prevent them from leaking into child processes during integration tests. <!-- [^9ac66-3] -->
