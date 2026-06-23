---
type: episode-card
date: 2026-06-09
session: 3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6.jsonl
salience: product
status: active
subjects:
  - wait-for-mention-command
  - agent-reactivity
supersedes: []
related_claims: []
source_lines:
  - 1-168
  - 212-258
captured_at: 2026-06-12T19:54:14Z
---

# Episode: wait-for-mention blocking command as idle-agent wake primitive

## Prior State

No mechanism for agents to receive mentions while idle; mentions were only consumed during active turns via the `inbox` command. An idle agent between prompts had no way to be woken by incoming messages.

## Trigger

User directive to add a command that blocks until a mention arrives, run in the background via the harness, so the agent is woken on completion. User corrected assistant's assumption that background process completion wouldn't wake an idle agent — verified that it does across harnesses.

## Decision

Implemented `tenex-edge wait-for-mention` subcommand: polls SQLite inbox every 500ms, performs relay self-fetch on startup (handles engine warmup race), prints drained mentions + re-run reminder on completion, exits 0. 5-minute default timeout prevents forgotten background processes. Agent re-runs it with `run_in_background=true` each time it completes.

## Consequences

- Idle agents are now woken by incoming mentions — a genuine coordination primitive for both active agents blocking on peer responses and idle agents awaiting contact
- The command reuses existing `fetch_mentions_into_inbox` + `drain_inbox` logic rather than adding new relay plumbing
- Creates a recurring agent behavior pattern: on mention receipt, act on it, then re-run the wait command

## Open Tail

- The re-run pattern depends on agent compliance (it reads the reminder and re-runs); no harness-level guarantee the agent will re-subscribe
- Timeout edge case: if a 5-min timeout fires with no mention, the agent gets a background-completion notification with no mention content

## Evidence

- transcript lines 1-168
- transcript lines 212-258

