---
type: episode-card
date: 2026-06-14
session: 0afc3cf4-3465-4b37-a7ec-63b798d78621
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/0afc3cf4-3465-4b37-a7ec-63b798d78621.jsonl
salience: product
status: active
subjects:
  - tmux-spawn-prompt
  - mention-delivery
supersedes: []
related_claims: []
source_lines:
  - 50-433
captured_at: 2026-06-18T00:28:36Z
---

# Episode: Spawn prompt replaced: actual mention message instead of generic 'tenex-edge inbox'

## Prior State

When tenex-edge tmux spawned a new agent session, it injected the hardcoded string `tenex-edge inbox` + Enter as the first prompt (via `SPAWN_PROMPT_DEFAULT`), regardless of whether the session was triggered by a mention or started manually. The intent was to force the agent to drain its inbox, but the text was semantically meaningless to the agent and fired even for manual TUI spawns.

## Trigger

User explicitly rejected the default prompt: 'no, that's a terrible idea -- if the session is being started because a new thread came in p-tagging an agent then tenex-edge tmux should start the session with whatever was received in the message -- not with some random "tenex-edge inbox" that is kinda totally meaningless for the agent!'

## Decision

Two-part behavior change: (1) Manual/TUI spawns now inject nothing — they start clean; `spawn_agent` no longer registers a default prompt. (2) Mention-triggered spawns now type the actual received message (rendered through the same envelope formatter the inbox uses, preserving sender/reply-id/body) as the first prompt, and mark the mention as already-delivered via a new `enqueue_mention_delivered` method so the turn-start drain won't duplicate it. Multiline messages are handled via tmux bracketed-paste (`paste-buffer -p`) instead of `send-keys -l`, which would submit after the first newline.

## Consequences

- Agents now see the actual p-tag message as their first prompt when spawned by a mention, giving them meaningful context immediately
- Manual TUI spawns start with a clean input box instead of a spurious 'tenex-edge inbox'
- Inbox reply threading (`inbox reply --id`) still works because the mention row persists with `find_inbox_by_event_prefix`, even though it's pre-marked delivered
- Bracketed-paste mode assumes the harness terminal supports it by the time the 2s spawn-delay elapses; non-Claude-Code harnesses may submit early if they don't enable bracketed-paste in time
- SPAWN_PROMPT_DEFAULT constant and `spawn_prompt` field on SpawnDef were removed/replaced

## Open Tail

- Bracketed-paste timing assumption needs live verification with non-Claude-Code harnesses (e.g., codex/opencode) — multiline messages could submit early if the harness hasn't enabled bracketed-paste mode by the 2s mark

## Evidence

- transcript lines 50-433

