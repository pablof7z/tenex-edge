---
type: episode-card
date: 2026-06-29
session: 661ebf6b-e01b-4ff6-b9c7-5042b900c788
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/661ebf6b-e01b-4ff6-b9c7-5042b900c788.jsonl
salience: product
status: active
subjects:
  - tenex-edge-who
  - hook-injection
  - fabric-context
supersedes: []
related_claims: []
source_lines:
  - 3713-3713
  - 3715-4030
captured_at: 2026-06-29T10:05:11Z
---

# Episode: Unify `tenex-edge who` output with hook injection fabric format

## Prior State

`tenex-edge who` rendered agent/channel information in markdown table format (`render_who_*`), distinct from hook injection format which used prose awareness snapshots.

## Trigger

User requested complete refactor of `who` output to structured agent-context format. Assistant implemented hook changes but left `who` command unchanged. User's correction at line 3713: 'did you update the formatting of tenex-edge who command? it looks exactly the same'.

## Decision

Unified both paths to single fabric format. Daemon `rpc_who` now renders via `render_fabric_view` (same as hook injection), returns as `fabric` string that CLI prefers over old snapshot table. Format: Project / Channel / Members (with self-marker) / Agents you can invite (roster).

## Consequences

- Breaking change for any automation parsing old table format — now renders prose with bullet lists instead of markdown tables
- Added `render_fabric_view` to handle unmaterialized channels (roots with no kind:39000 metadata) — returns rendered string instead of None
- Changed fallback label for unmaterialized scopes from work-title (would hijack session task names) to scope-id (project slug for roots)
- Unified code path eliminates duplication between `who` command and hook renderers

## Open Tail

- Colorization not applied to plain-text output (works in TTY only)
- `--all-projects` flag still uses old table format (not yet converged)

## Evidence

- transcript lines 3713-3713
- transcript lines 3715-4030

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-unify-tenex-edge-who-output-with.json`](transcripts/2026-06-29-1-unify-tenex-edge-who-output-with.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-unify-tenex-edge-who-output-with.json`](transcripts/raw/2026-06-29-1-unify-tenex-edge-who-output-with.json)
