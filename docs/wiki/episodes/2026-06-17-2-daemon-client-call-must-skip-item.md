---
type: episode-card
date: 2026-06-17
session: 52474db7-1e81-4011-a859-6343bfeae807
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/52474db7-1e81-4011-a859-6343bfeae807.jsonl
salience: architecture
status: active
subjects:
  - daemon-rpc-protocol
  - client-call
supersedes: []
related_claims: []
source_lines:
  - 554-575
  - 762-792
  - 1113-1131
captured_at: 2026-06-18T00:51:52Z
---

# Episode: Daemon client call() must skip item progress frames

## Prior State

The synchronous call() method in daemon/client.rs read a single frame from the daemon and expected it to be an ok or error response. It would fail with "daemon returned neither ok nor error" if any other frame type arrived first.

## Trigger

session_start was recently changed to emit item progress frames (init_progress) before the terminal ok response. The call() method, used by the blocking CLI path and integration tests, received these item frames first and panicked because it didn't recognize them as terminal responses.

## Decision

call() now loops reading frames, skipping item-type frames until it receives a terminal ok or error frame, matching the pattern already established by call_with_items() for async contexts.

## Consequences

- The blocking CLI path (hook, who, etc.) no longer crashes when session_start emits progress frames
- Integration test env isolation was also fixed: TENEX_EDGE_AGENT_FALLBACK is now stripped from subprocess env alongside TENEX_EDGE_AGENT, preventing the live shell's 'developer' slug from leaking into tests

## Open Tail

*(none)*

## Evidence

- transcript lines 554-575
- transcript lines 762-792
- transcript lines 1113-1131

