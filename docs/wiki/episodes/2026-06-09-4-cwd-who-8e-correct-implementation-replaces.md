---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: product
status: active
subjects:
  - who-output-format
  - rel-cwd
  - presence-status-wire
  - remote-annotation
supersedes: []
related_claims: []
source_lines:
  - 863-876
  - 892-898
  - 1037-1100
captured_at: 2026-06-17T23:48:53Z
---

# Episode: cwd/who §8e: correct implementation replaces buggy partial

## Prior State

The who output was half-implemented by another agent: no cwd/rel-path at all (the whole point of seeing which worktree a session is in was missing), same-machine sessions wrongly tagged (remote on hostname), and inconsistent (some sessions got no annotation while others did).

## Trigger

Observing the buggy who output post-daemon-cutover: this session (162f9965) on this machine was mislabeled '(remote on pablos-laptop)', other same-machine sessions had no annotation at all, and no rel_cwd was shown. The correct design was already spec'd in docs/daemon-design.md §8e (marked DEFERRED).

## Decision

Implement §8e as specified: rel_cwd (project-relative working directory) added to Presence/Status wire (optional ['rel-cwd', <rel>] tag on kind:30315, backward-compatible), persisted in sessions/peer_sessions tables (idempotent ALTER TABLE ADD COLUMN). who output is now two-line format with [rel_cwd] bracket (omitted for empty/.) and (remote) only for genuinely remote peers. Same-host peers get no annotation.

## Consequences

- who stdout contract changed (two lines per agent + [rel_cwd] + (remote)) — anything parsing who may need updating
- Wire protocol change is backward-compatible (optional tag, decode tolerates absence → empty string)
- git worktree dirs resolve to their own path via git rev-parse --show-toplevel, so worktree1/worktree2 both render as '.' unless a .tenex/project.json sits at their common parent
- The buggy local_host_slug/remote_host_annotation code (which did a fragile second Config::load()) was deleted

## Open Tail

- Git worktrees both resolving to '.' — needs .tenex/project.json at common parent to distinguish them

## Evidence

- transcript lines 863-876
- transcript lines 892-898
- transcript lines 1037-1100

