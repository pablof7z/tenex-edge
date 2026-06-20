---
title: tenex-edge `who` Command Implementation
slug: tenex-edge-who-command-implementation
topic: tenex-edge
summary: The `src/cli/who.rs`, `src/cli/who/render.rs`, and `src/cli/who/tests.rs` files (821 lines total) are dead code â they are never declared with `mod who;` in `
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-16
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:4ba07cd0-c4df-4e63-ae13-90c20c46f6ce
  - session:a88513d3-754f-4369-b440-72c8d29331e2
  - session:rollout-2026-06-09T10-55-30-019eab61-23ae-7163-8d06-9a3965847e4f
  - session:rollout-2026-06-09T12-56-40-019eabd0-1205-77a3-88b8-e07b0d948f1d
  - session:rollout-2026-06-09T15-01-20-019eac42-32f0-7ff0-bda2-da2de3b78ed7
  - session:rollout-2026-06-12T11-18-49-019ebae9-8fa7-73f1-844d-bea23bfb0193
  - session:ses_13a5107b0ffeS3nHRuWFcAx21V
---

# tenex-edge `who` Command Implementation

## Dead Code: `src/cli/who.rs`

The `src/cli/who.rs`, `src/cli/who/render.rs`, and `src/cli/who/tests.rs` files (821 lines total) are dead code — they are never declared with `mod who;` in `src/cli.rs` and are never compiled. The live `who` implementation resides inline in `src/cli.rs`.

<!-- citations: [^4ba07-2] [^ses_1-21] -->
## Output Renderers

The `tenex-edge who` output has separate renderers for human and agent consumers, auto-selected by TTY detection: terminal uses the human format; piped/captured uses the agent markdown format. The human renderer produces colorized, column-aligned output with bold section headers, a Sessions table, agents on one line, and Other projects as a dotted list. The `who` command renders the current project as a heading followed by one line per active session in the format `<agent> [session <id>] - <status>`, with no `agents:` label or `@project` suffix on entries for the current project. The project slug suffix is omitted only when the agent entry is within the same project context; it is always shown in cross-project or global listings. It displays an "other agent(s) in other projects" section listing unique agent slugs with project suffixes (`agent@projectSlug`), deduplicating by agent identity per project. When `who --all-projects` is used, every row includes `agent@project` because there is no single current-project heading to provide implicit context. The agent renderer produces no ANSI, uses markdown headings (`# Sessions`, `# Agents (for new sessions)`, `# Other projects`), a markdown table (`| Agent | Session | Host | Title | Status |`), and embeds the actual `inbox send` and `new-session` commands. The agent renderer uses an em-dash (—) for empty session titles instead of leaving the cell blank. The one-shot `who` output displays `agent@project` while the live board displays `agent@host`. The `who` command hides same-host relay echoes (peer_sessions) for agent pubkeys that match a known local agent identity with no live local session.

<!-- citations: [^4ba07-2] [^a8851-3] [^rollo-13] [^rollo-17] [^rollo-23] [^rollo-43] -->
## Output Structure

The `who` output includes the project name as `Project: <name>`. The section order is: Sessions, Agents (for new sessions), Other projects. The Sessions section includes an explainer: "Message an active session with `tenex-edge inbox send --to-session <codename> --subject ... --message ...`." The Agents section includes an explainer: "Start a new session with `tenex-edge inbox send --to-new-session <slug> --subject ... --message ...`", and the old `[spawnable via ...]` tags are removed from agent entries. The Other projects section lists project names only, dropping the project `about`/description that the old format rendered. <!-- [^a8851-4] -->

## New Session Command

Starting a new session with a spawnable agent is done via `tenex-edge inbox send --to-new-session <slug> [--project <slug>] --subject ... --message ...`: it spawns a fresh harness for that agent (project defaulting to the current working directory) and delivers the message to the new session. <!-- [^a8851-5] -->

## Hidden Details

The `developer` agent's `--dangerously-skip-permissions` harness detail is no longer shown anywhere in the `who` output. <!-- [^a8851-6] -->

## Live TUI Board

The `tenex-edge who --live` command opens a full-screen TUI board that continuously refreshes, showing AGENT@HOST, project, status, session, and seen-age columns, and exits cleanly on `q`, Esc, or Ctrl-C. The `--live` flag supports `--all` to keep stale sessions visible and `--refresh-ms` to control the refresh interval. <!-- [^rollo-12] -->

The `--live` renderer uses `\r\n` line endings in raw terminal mode so each rendered line returns to column 0. <!-- [^rollo-18] -->

## Messaging and Routing

Sending a message to a bare agent name (e.g., "codex") auto-resolves it as `<agent>@$currentProject` so the message routes within the correct project scope. Project-scoped agent lookup is strict: it does not fall back through a global profile before project-scoped presence, so a global profile cannot override project presence. Local delivery routes messages by `(pubkey, project)`, ensuring that an agent with sessions in multiple projects only receives the message in the addressed project. <!-- [^rollo-24] -->

## Protocol and Compatibility

The daemon protocol version is bumped when the `who` snapshot shape changes, forcing stale daemons to exit and respawn rather than serving incompatible responses. Deserialization of `other_projects` snapshot data is backward-compatible: missing fields (e.g., `agents`) do not crash a client encountering a stale daemon response during transition. <!-- [^rollo-25] -->
