---
type: episode-card
date: 2026-06-17
session: d8d132f9-8a71-4af0-846c-44a4a9e01dc5
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d8d132f9-8a71-4af0-846c-44a4a9e01dc5.jsonl
salience: architecture
status: active
subjects:
  - agent-identity-schema
  - harness-spawn-path
  - per-harness-translation
supersedes:
  - 2026-06-14-1-spawnable-agents-source-of-truth-identity
related_claims: []
source_lines:
  - 96-96
  - 267-276
  - 285-303
  - 392-397
  - 465-484
  - 543-554
  - 570-577
captured_at: 2026-06-18T00:49:02Z
---

# Episode: Agent spawn uses inline agent definitions with per-harness translation

## Prior State

Agent spawn relied on the `command` field in identity files to carry the full launch command, including harness-specific flags like `--agent <path-to-definition-file>`. Agent definitions lived in separate files under `agents-definition/`. The spawn system had no concept of per-harness command translation.

## Trigger

User found that `--agent <path>` doesn't properly launch agents in claude code and proposed embedding the agent definition directly in the identity file with a per-harness translator that expands `--agents '{<slug>: <def>}' --agent <slug>` for claude. This was preceded by diagnosing that the `command` field was a plain string (not array), causing silent serde deserialization to None, and that `~` in paths doesn't expand because tmux receives commands directly without a shell.

## Decision

Added `agent: Option<serde_json::Value>` to `StoredKey` to store agent definitions inline. Created `apply_agent_def_args` as a per-harness translator: for `claude` binary it wraps the def as `{<slug>: <def>}` and appends `--agents '<json>' --agent <slug>`; other harnesses are a no-op. The `command` field now holds only the base harness command (e.g., `["claude", "--dangerously-skip-permissions"]`). `list_local_agents` returns a 3-tuple `(slug, command, agent_def)`. `resolve_spawn_entry` replaces `resolve_agent_command`.

## Consequences

- Agent definitions are now self-contained in identity files, eliminating the need for separate `agents-definition/` files
- Per-harness translation layer exists; only claude is implemented, other harnesses pass through unchanged
- Spawn and resume paths diverge: spawn applies `apply_agent_def_args`, resume uses base command only
- The `command` field must be an array — a plain string causes silent deserialization to None
- Tilde (`~`) in command paths won't expand since tmux receives commands without a shell

## Open Tail

- Tilde expansion could be added in `resolve_spawn_entry` for portability
- Other harnesses (codex, opencode) may need their own translation logic in `apply_agent_def_args`
- The claude translator maps `system_prompt` → `prompt` — field name mapping is harness-specific and may need updating if claude's CLI changes

## Evidence

- transcript lines 96-96
- transcript lines 267-276
- transcript lines 285-303
- transcript lines 392-397
- transcript lines 465-484
- transcript lines 543-554
- transcript lines 570-577

