---
type: episode-card
date: 2026-06-12
session: da7ab617-89fb-4b68-9e2d-3f251fe6c1d9
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/da7ab617-89fb-4b68-9e2d-3f251fe6c1d9.jsonl
salience: product
status: superseded
subjects:
  - tenex-edge-who-remote-display
supersedes: []
related_claims: []
source_lines:
  - 1-192
captured_at: 2026-06-18T00:08:48Z
---

# Episode: who command: show hostname instead of generic (remote) tag

## Prior State

The `who` command rendered remote agents with a static `(remote)` annotation, hiding the actual machine identity behind a generic label.

## Trigger

User reported that `(remote)` provides no information about which computer the remote agent is running on.

## Decision

Changed `render_who_row` and `render_who_plain` to display the peer's actual hostname — e.g. `(tower)` — instead of the literal `(remote)` string, using the `host` field already present on `WhoRow`.

## Consequences

- Users can now identify which specific machine a remote agent is running on.
- Tests that asserted the literal `(remote)` string were updated to assert the hostname value instead (e.g. `(tower)`).
- The §8e comment in render.rs was updated to reflect the new semantics: `gets (hostname)` rather than `gets (remote)`.

## Open Tail

*(none)*

## Evidence

- transcript lines 1-192

