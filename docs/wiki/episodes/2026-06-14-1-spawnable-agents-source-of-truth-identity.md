---
type: episode-card
date: 2026-06-14
session: bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4.jsonl
salience: architecture
status: superseded
subjects:
  - spawnable-agents
  - identity-store
  - who-snapshot
supersedes: []
related_claims: []
source_lines:
  - 1-4
  - 22-50
  - 89-100
  - 122-142
  - 971-1003
  - 1188-1262
  - 1664-1732
  - 1811-1856
captured_at: 2026-06-18T00:18:47Z
---

# Episode: Spawnable agents source of truth: identity store replaces PATH

## Prior State

spawnable_agents() used a hardcoded SPAWN_DEFS table checked against $PATH to determine which agent harnesses were available; agent identity files stored only keypairs without harness commands; spawnable population code lived in src/cli/who.rs — a dead path the daemon never called

## Trigger

User reported 'spawnable no sessions' UI label; investigation revealed the daemon's rpc_who calls load_who_snapshot in src/cli.rs (which had zero spawnable logic), while the spawnable code in src/cli/who.rs was unreachable; additionally, agent identity files had no command field to carry harness invocations like 'claude --dangerously-skip-permissions'

## Decision

spawnable_agents() now reads from identity::list_local_agents() instead of SPAWN_DEFS+PATH; agent identity files gained command: Option<Vec<String>> field (e.g. developer.json now stores its own harness command); SPAWN_DEFS retained only as fallback for legacy agents without a command field; SpawnableRow and spawnable population added to the correct WhoSnapshot struct in src/cli.rs (the daemon code path)

## Consequences

- Agent slugs like 'developer' carry custom harness commands from their identity file rather than relying on PATH discovery
- The spawnable section now appears in daemon-served who output and the TUI
- SPAWN_DEFS is demoted to legacy fallback — new agents define their command in the identity file
- Two separate WhoSnapshot structs remain (src/cli.rs and src/cli/who.rs); only the cli.rs one is authoritative

## Open Tail

- The cli/who.rs WhoSnapshot and SpawnableRow are now dead code and could be removed

## Evidence

- transcript lines 1-4
- transcript lines 22-50
- transcript lines 89-100
- transcript lines 122-142
- transcript lines 971-1003
- transcript lines 1188-1262
- transcript lines 1664-1732
- transcript lines 1811-1856

