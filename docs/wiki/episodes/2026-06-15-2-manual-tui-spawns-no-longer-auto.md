---
type: episode-card
date: 2026-06-15
session: 622711fa-5176-4580-b311-d66446c2924b
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/622711fa-5176-4580-b311-d66446c2924b.jsonl
salience: architecture
status: active
subjects:
  - session-spawn-init
  - pending-spawn-registry
supersedes: []
related_claims: []
source_lines:
  - 3-5
  - 92-107
  - 636-664
  - 836-865
captured_at: 2026-06-18T00:31:47Z
---

# Episode: Manual TUI spawns no longer auto-inject 'tenex-edge inbox' prompt

## Prior State

Every manual TUI spawn registered a PendingSpawn with SPAWN_PROMPT_DEFAULT = 'tenex-edge inbox', causing rpc_session_start to inject that text as the first user message ~2 seconds after startup regardless of whether any messages existed.

## Trigger

User report: 'when I create a new session with tenex-edge tmux it still sends, after a few seconds, "tenex-edge inbox" as a user message — it shouldn't send ANYTHING!'

## Decision

Removed SPAWN_PROMPT_DEFAULT entirely; manual TUI spawns now register no PendingSpawn entry, so rpc_session_start injects nothing and the session starts clean. Only spawn-on-send (inbound mention triggering a new agent) registers a PendingMention and gets its message typed in as the first prompt.

## Consequences

- Manual TUI spawns start with a blank prompt — no automatic 'tenex-edge inbox' command
- Spawn-on-send flow is preserved: mentions that trigger a new agent still inject the received message as the first prompt
- The doorbell mechanism (injecting 'You have new tenex-edge mentions…' when unread inbox rows exist) remains unchanged and is the only path that sends 'tenex-edge inbox' text into a pane

## Open Tail

*(none)*

## Evidence

- transcript lines 3-5
- transcript lines 92-107
- transcript lines 636-664
- transcript lines 836-865

