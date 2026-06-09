---
title: Tenex-Edge Messaging
slug: tenex-edge-messaging
topic: tenex-edge
summary: Sending a message to another agent uses `tenex-edge send-message` accepting either an agent slug via `<agentSlug>@<projectSlug>` or a session ID via `--recipien
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-09
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:ses_15544a0c8ffeTRok1tpY00hCS9
  - session:ses_1554673ecffeiKUCnZUlYuA7Zw
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:ses_154516e41ffeZc8cdD1RWFtUul
  - session:240ffb86-8827-4741-932b-29fb1824c0c7
---

# Tenex-Edge Messaging

## Agent Messaging

The term for inter-agent messages is 'mention', not 'direct message'. Mentions are sent via `tenex-edge send-message` with an `agentSlug@projectSlug` recipient format, publishing as a kind:1 event with a NIP-29 `h` tag for the project group and a p-tag for the destination agent's pubkey. Mentions can target a particular session when the same agent is running in multiple sessions, using `--recipient <session-id>`; `--recipient` also accepts an agent name when no session is running. Send-message resolves the recipient by session-id prefix across both peer sessions and own sessions. When mentioning an agent's kind:0, the nostr EventBuilder must use `.allow_self_tagging()` so that p-tags equal to the author's pubkey are preserved (required for same-agent cross-session messaging). Mentions are deduplicated per-agent (not per-session); once an agent has seen a mention in any session, it never resurfaces in a later session. A `tenex-edge tail -f` command streams all messages colorized, optionally filtered by project slug; in presence events, the hostname appears in the `@` slot and the project name is shown dimmed in parentheses. The UserPromptSubmit hook injects the available agents list (who) with what each is doing, so agents automatically know who's reachable without running a command. The `who` command displays agents across all projects by default, regardless of which project directory it is run from; passing `--project <slug>` filters the view to a specific project. When agents span multiple projects, the `who` output groups or adds section headers by project to make the cross-project scope explicit. The agent display format uses `agentSlug@hostname`, where the hostname is sourced from the `backendName` property in the `.tenex/config.json` file and must be sluggified to prevent ambiguity when agents address messages to one another; the raw hostname value is preserved unmodified in the Nostr data model and Nostr tags, and slugification is applied only at display time. In the `who` command's compact view, the agent's project is displayed in the secondary dim field previously occupied by the host. In the `who` command's live table view, `slug@host` is shown in the AGENT column and the separate HOST column is removed. (Previously: the agent display format used `agentSlug@hostname` where the hostname was sourced from `backendName`, but `agentSlug@project` was used in the recipient format.) Agents can run `tenex-edge who`, `inbox`, or `send-message` without specifying `--session`; the session is resolved via the `$TENEX_EDGE_SESSION` environment variable (exported by launchers/plugins) or the current working directory's project. Claude Code and other harnesses have access to a send-message skill to send messages to other agents. The send-message skill injects `CLAUDE_SESSION_ID` as a documented skill substitution because that id is not available in the Bash environment. The inbox command self-fetches stored mentions from the relay so receive works even in one-shot runs where the engine hasn't had time to accumulate them. A wait-for-mention command polls the SQLite inbox for incoming mentions, prints them upon receipt, and then exits. Its output includes a reminder to re-run `tenex-edge wait-for-mention` with `run_in_background=true` to receive the next mention. The agent runs `wait-for-mention` via its shell with `run_in_background=true` rather than using shell `&`, so the harness tracks the process and wakes the agent on completion. An idle agent is woken when a background process it launched completes, enabling the `wait-for-mention` flow to work for idle agents. The `wait-for-mention` command performs the same relay self-fetch as `inbox` on startup to handle the engine warmup race, polls the SQLite inbox every 500ms, and has a default timeout of 5 minutes to prevent forgotten background processes from lingering. The `wait-for-mention` instruction is injected via the `UserPromptSubmit` hook, firing exactly once per session (tracked via a temp flag file keyed on `sid`), so the agent can act on it during its first active turn. The session-start hook is not a suitable place for this instruction because no LLM call occurs at session start; the agent is idle and cannot execute commands until a user prompt triggers a turn. The relay `relay.tenex.chat` requires NIP-42 AUTH for reads; subscriptions opened before auth completes get silently closed. Transport::connect forces an auth warm-up fetch before any subscribe.

<!-- citations: [^f3a73-71] [^f3a73-31] [^f3a73-41] [^f3a73-53] [^f3a73-64] [^f3a73-70] [^f3a73-76] [^f3a73-82] [^f3a73-90] [^ses_1-1] [^ses_1-2] [^3da7f-1] [^f3a73-101] [^f3a73-107] [^3da7f-5] [^ses_1-3] [^240ff-1] [^240ff-4] -->
## Milestone Scope

Inbound injection of peer messages is included in M1 scope. Message injection is immediate (not deferred).

<!-- citations: [^f3a73-33] [^f3a73-55] [^f3a73-72] [^f3a73-89] [^f3a73-102] -->
