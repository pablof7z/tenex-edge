---
type: episode-card
date: 2026-06-16
session: 7cac50b6-a19d-4bd8-9be7-5c52aa8b2cca
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/7cac50b6-a19d-4bd8-9be7-5c52aa8b2cca.jsonl
salience: architecture
status: active
subjects:
  - agent-identity
  - kind0-profile
  - network-discovery
supersedes: []
related_claims: []
source_lines:
  - 470-723
captured_at: 2026-06-18T00:46:39Z
---

# Episode: Kind:0 profile publishing on agent creation

## Prior State

Profile (kind:0) was only published as a side-effect of an agent running a session (the engine publishes it on session-start). A created-but-never-run agent was invisible to profile discovery (NIP-05, kind:0 lookups).

## Trigger

User directive: 'make sure that any agent creation publishes its kind:0 as expected'

## Decision

Added a `publish_profile` daemon RPC that loads the agent's keys from the keystore by slug, builds the same DomainEvent::Profile the engine does (content: {name: slug}, tags: [host, owner]), and publishes it via the daemon's provider/transport pool. `agent add` calls this RPC only for newly created keypairs (re-running to update a command does not re-publish). If the daemon is unavailable, the create still succeeds with a 'profile publish deferred to first session' message — the engine re-publishes the identical Profile on first run.

## Consequences

- Newly created agents are immediately discoverable via profile lookup without needing to run a session first
- The profile event is signed by the agent's own keys (not the operator's), consistent with session-start publishing
- Best-effort publishing means offline daemons don't block agent creation — the invariant holds that kind:0 will eventually appear
- agent assign deliberately does NOT re-publish the profile (the agent was already created and published)

## Open Tail

*(none)*

## Evidence

- transcript lines 470-723

