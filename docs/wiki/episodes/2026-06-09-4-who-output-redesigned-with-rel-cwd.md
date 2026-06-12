---
type: episode-card
date: 2026-06-09
session: 162f9965-82ca-420b-aa24-99faa15cb59a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/162f9965-82ca-420b-aa24-99faa15cb59a.jsonl
salience: product
status: active
subjects:
  - tenex-edge
  - who-command
  - presence-wire-protocol
supersedes: []
related_claims: []
source_lines:
  - 863-876
  - 892-898
  - 1037-1099
captured_at: 2026-06-12T20:02:14Z
---

# Episode: who output redesigned with rel_cwd and correct remote annotation

## Prior State

who showed single-line output per agent; same-machine sessions were wrongly tagged '(remote on pablos-laptop)'; no working directory information was displayed; the (remote) annotation used a fragile second Config::load().

## Trigger

User requested cwd in presence/who (§8e). A parallel agent's partial implementation was buggy — same-host sessions mislabeled 'remote', no cwd shown at all, inconsistent annotation.

## Decision

Two-line who format: line 1 = agent@project [session] [rel_cwd] (remote), line 2 = status. rel_cwd is project-relative (project_root from .tenex/project.json or git rev-parse --show-toplevel; '.'/omitted for root, 'src' for subdir). remote is computed daemon-side (source==Peer && slugify(peer.host) != slugify(daemon_host)); local rows always remote=false. Wire: optional tag ['rel-cwd', <rel>] on kind:30315 presence+status events, backward-compatible. Schema: idempotent ALTER TABLE … ADD COLUMN rel_cwd.

## Consequences

- who stdout contract changed — anything parsing it (channel adapter, integrations) may need updating.
- Real git worktree dirs: git rev-parse --show-toplevel returns each worktree's own path, so worktree1/worktree2 both resolve to '.' unless a .tenex/project.json sits at their common parent.
- The fragile local_host_slug/remote_host_annotation code (second Config::load()) was deleted entirely.
- 81 tests green including new render tests (same-host-peer-not-remote, root-cwd-no-bracket).

## Open Tail

*(none)*

## Evidence

- transcript lines 863-876
- transcript lines 892-898
- transcript lines 1037-1099

