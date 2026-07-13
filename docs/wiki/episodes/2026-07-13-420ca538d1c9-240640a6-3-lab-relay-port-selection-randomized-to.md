---
type: episode-card
date: 2026-07-13
session: 420ca538-d1c9-4af5-91fc-3e634d2d8442
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/420ca538-d1c9-4af5-91fc-3e634d2d8442.jsonl
salience: workflow
status: active
subjects:
  - start-croissant-relay
  - port-selection
  - lab-skill
supersedes: []
related_claims: []
source_lines:
  - 1614-1716
captured_at: 2026-07-13T08:15:49Z
---

# Episode: Lab relay port selection randomized to eliminate concurrent-lab collisions

## Prior State

The `start-croissant-relay` lab script used `pick_port()` which scanned upward from a fixed base (19888) and returned the lowest free port. Every lab deterministically gravitated to the same low ports, and a TOCTOU race between the `lsof` check and croissant binding meant two labs could both see 19888 free and collide at the NIP-29 group-ownership layer.

## Trigger

During the pty-wrap-me live smoke test, the first relay came up on the default shared port 19888 and collided with another agent from the running fleet (a `claude-acp` daemon) that had already created the NIP-29 `workspace` group. The lab's keys were `blocked: unknown member` for kind:9. User asked whether the e2e harness should just use a random port.

## Decision

Changed `pick_port()` to randomize the base across 20000–50000 by default (`20000 + RANDOM % 30000`), with up to 5 fresh random-base retries on TOCTOU collision. Env overrides preserved: `TENEX_EDGE_DEV_RELAY_PORT_BASE` pins a base, `TENEX_EDGE_DEV_RELAY_PORT` forces an exact port. A pinned base still fails fast if fully occupied rather than looping.

## Consequences

- Concurrent labs and the live fleet now practically never share a relay port, killing the NIP-29 group-ownership collision at its root.
- The TOCTOU race between `lsof` check and binding is mitigated by re-rolling the base on collision.
- Change is in the skill scripts (`~/.claude/skills/tenex-edge-dev/scripts/start-croissant-relay`), not the repo — no PR needed.
- Verified: three consecutive runs picked 44864, 47590, 33477; pinned base 30500 honored; full script passes `bash -n`.

## Open Tail

*(none)*

## Evidence

- transcript lines 1614-1716

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-13-420ca538d1c9-240640a6-3-lab-relay-port-selection-randomized-to.json`](transcripts/2026-07-13-420ca538d1c9-240640a6-3-lab-relay-port-selection-randomized-to.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-13-420ca538d1c9-240640a6-3-lab-relay-port-selection-randomized-to.json`](transcripts/raw/2026-07-13-420ca538d1c9-240640a6-3-lab-relay-port-selection-randomized-to.json)
