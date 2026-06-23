---
type: episode-card
date: 2026-06-16
session: 1b868736-ed6b-4f88-84d9-26bb320accfd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/1b868736-ed6b-4f88-84d9-26bb320accfd.jsonl
salience: product
status: active
subjects:
  - title-seed
  - turn-start-rpc
  - user-prompt-submit-hook
supersedes:
  - 2026-06-16-2-session-title-appears-immediately-and-distills
related_claims: []
source_lines:
  - 618-995
captured_at: 2026-06-16T11:11:54Z
---

# Episode: Thread prompt text into turn_start to eliminate title-seed lag

## Prior State

The user-prompt-submit hook had the prompt text but only used it for the kind:1 Nostr publish. turn_start received only the transcript file path. The runtime seed (runtime.rs:258-270) called read_last_user_prompt(transcript), but Claude Code hadn't flushed the just-submitted prompt to the transcript file yet at turn_start time. Result: first message published an empty title; every subsequent message showed the previous message's title.

## Trigger

User reported: 'I send the first message to claude code, it doesn't publish ANY title (it does publish the heartbeat, but it's empty) — once I send a second message is when it starts publishing a title (which is the first message I sent)'

## Decision

Added a last_user_prompt column to the sessions table and a set_/get_last_user_prompt API. The prompt text is now captured verbatim in the user-prompt-submit hook, threaded through turn_start (hooks.rs → turn.rs → rpc_turn_start), persisted in the DB at turn-start time, and the runtime seed prefers it over the lagging transcript read.

## Consequences

- Titles appear immediately on the first user message
- Title no longer lags one message behind
- The transcript read is kept as a fallback path if the prompt capture is empty

## Open Tail

*(none)*

## Evidence

- transcript lines 618-995

