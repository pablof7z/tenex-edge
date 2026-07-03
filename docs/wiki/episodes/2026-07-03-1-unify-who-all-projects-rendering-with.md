---
type: episode-card
date: 2026-07-03
session: 7d6bf2fe-8dc9-4bd0-aeeb-de1827bf68d1
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/7d6bf2fe-8dc9-4bd0-aeeb-de1827bf68d1.jsonl
salience: product
status: active
subjects:
  - tenex-edge-who
  - fabric-context
  - unified-rendering
supersedes:
  - 2026-06-29-1-unify-tenex-edge-who-output-with
related_claims: []
source_lines:
  - 1-63
  - 547-612
  - 630-671
  - 930-963
  - 1086-1090
captured_at: 2026-07-03T11:40:08Z
---

# Episode: Unify `who --all-projects` rendering with fabric context pipeline

## Prior State

`tenex-edge who` (single-project) rendered via the unified fabric context pipeline (`build_view` → `render_view`/`render_human_view`), but `who --all-projects` fell back to the old `WhoSnapshot`/`render_who_*` markdown-table renderer because the daemon's `rpc_who` only built the fabric view when a single project scope resolved. The `--all-projects` branch skipped fabric rendering entirely — a known open tail from the 2026-06-29 unification session.

## Trigger

User observed at line 1 that `who --all-projects` was 'seriously using a completely different renderer' and stated 'it obviously fucking shouldn't!'

## Decision

Added `render_fabric_all_projects`/`_human` entry points in `fabric_context.rs` (rendering one project block per root channel with roster up front) and wired them into the `all_projects` branch of `rpc_who` in the daemon. All `who` variants — single-project, `--all-projects`, and hook-injection turn context — now go through the single `build_view` → `render_view`/`render_human_view` pipeline.

## Consequences

- The old `src/cli/who/render.rs` + `src/who_snapshot.rs` markdown-table renderer is now purely a fallback for version-skewed daemons that don't emit a `fabric`/`fabric_human` string, not an active duplicate on any primary code path.
- Hook injection (turn-start, PostToolUse), single-project `who`, and `--all-projects` `who` all share one rendering pipeline — no rendering drift is possible between them.
- Existing unit and integration tests (8 fabric_context tests, 17 who/rendering tests, 7 daemon integration tests) all pass with the change.
- New integration test `who_all_projects_uses_unified_fabric_render_not_old_table` was added but had a wrinkle around session-id anchoring for the second project block (test initially failed because the sandbox only had one project).

## Open Tail

- Integration test `who_all_projects_uses_unified_fabric_render_not_old_table` needs fixing — it panicked because the test sandbox only created one project so the multi-project fabric block wasn't exercised as expected.
- A transient build break in `chat_write.rs`/`util.rs` (another agent's uncommitted edit) blocked final test runs; needs retry once that settles.

## Evidence

- transcript lines 1-63
- transcript lines 547-612
- transcript lines 630-671
- transcript lines 930-963
- transcript lines 1086-1090

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-03-1-unify-who-all-projects-rendering-with.json`](transcripts/2026-07-03-1-unify-who-all-projects-rendering-with.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-03-1-unify-who-all-projects-rendering-with.json`](transcripts/raw/2026-07-03-1-unify-who-all-projects-rendering-with.json)
