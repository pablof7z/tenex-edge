---
type: episode-card
date: 2026-06-16
session: 412e32c5-05f9-4e2a-86c6-e1c21e464553
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/412e32c5-05f9-4e2a-86c6-e1c21e464553.jsonl
salience: root-cause
status: superseded
subjects:
  - daemon-startup-sequence
  - socket-lifecycle
  - cold-start-latency
supersedes: []
related_claims: []
source_lines:
  - 221-243
captured_at: 2026-06-16T09:44:13Z
---

# Episode: Slow cold-start caused by socket-bind-before-accept-loop gap

## Prior State

Daemon startup was assumed to be reasonably fast; the socket bind and accept loop ordering was not recognized as a latency bottleneck.

## Trigger

User reported that tenex-edge tmux takes a really long time to start. Investigation revealed that on cold start, the daemon binds the Unix domain socket (server.rs line 131) before spawning the accept loop (line 208), with Transport::connect + NIP-42 AUTH warmup (up to 8+ seconds) in between. The client connects to the socket immediately, sends hello, then blocks waiting for a welcome that can't be served until the accept loop starts.

## Decision

Root cause identified: socket bind must not precede accept-loop readiness. Two fix options proposed — (1) move Transport::connect before bind_socket so the socket only appears once the daemon is fully ready, or (2) start the accept loop early and queue/defer requests until relay warmup completes.

## Consequences

- Cold-start latency of 8+ seconds is explained entirely by the bind-before-accept sequencing; warm starts are unaffected.
- Any fix must preserve the flock-protected race safety of spawn_if_absent.
- Option 1 (reorder) is simplest with no behavior change; Option 2 (early accept) adds complexity but allows overlapping client handshakes with relay warmup.

## Open Tail

- Which fix option to implement has not been chosen yet.
- No code change has been made in this session.

## Evidence

- transcript lines 221-243

