---
type: research-record
date: 2026-07-09
session: b70718e1-7221-496a-bc81-1ae5d1706296
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b70718e1-7221-496a-bc81-1ae5d1706296.jsonl
source_lines: 4274-4323
agent_attribution: a394ed0ea4e0a487f
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Full project-wording purge report: 184-file rename verified via 890 unit + 46 integration tests and grep confirmation of zero 'project' in human-facing surfaces — verdict: complete, all gates green"
captured_at: 2026-07-09T18:34:45Z
---

Full project-wording purge report: 184-file rename verified via 890 unit + 46 integration tests and grep confirmation of zero 'project' in human-facing surfaces — verdict: complete, all gates green

---

User: [Agent task result: Agent "Full project-wording purge" finished]
All old files removed, new files present, 184 files staged. Every gate passes. The purge is complete.

## Report

Purged the "project" vocabulary from the `tenex-edge` codebase (worktree `/Users/pablofernandez/src/tenex-edge-purge`, branch `refactor/purge-project-wording`), splitting it into two concepts per the naming map: **workspace** (machine+path binding) and **root channel** (parent-empty top-of-tree channel). ~184 files changed, all changes staged and uncommitted.

### Green gates (all pass)
- `cargo build` — clean
- `just fmt-check` — clean
- `just lint` (clippy `-D warnings`) — clean
- `just loc-check` — all files under 500 hard limit; **no new soft-limit drift** vs `origin/master` (77400d3f)
- `just test-unit` — **890 passed, 0 failed**
- `cargo test --test daemon_integration -- --test-threads=1` — **46 passed, 0 failed** (the heavy e2e suite via local `nak` relay)

### Requirement 1 — rendered human-facing context has NO "project" (verified)
`grep` for `<project`, `"Project:`, `project-relative`, `<no-new-activity project` across `src/` returns **NONE**. The hook wrapper now emits `<workspace name="X">` (`src/fabric_context/render.rs:14`). The second render path (`turn_context/start.rs` warning) renders `channel "X"` with an internal `root_name` var. Statusline/who now use `Root:` / `Other root channels` / `all root channels`.

### Files renamed (git mv, staged as renames where detected)
- `src/project.rs` → `src/workspace.rs` (+ split tests to `src/workspace/tests.rs` to satisfy loc-check on the rename)
- `src/state/project_roots.rs` → `src/state/workspace_roots.rs`
- `src/daemon/server/rpc/project.rs` → `src/daemon/server/rpc/channel_members.rs`
- `src/daemon/server/probe/validate/project_root.rs` → `.../workspace.rs`
- `src/daemon/server/probe/tests/validate_project_root.rs` → `.../validate_workspace.rs`
- `src/cli/probe/validate_render/project_root_tail.rs` → `.../workspace_tail.rs`
- `src/cli/probe/validate_render/tests/project_root.rs` → `.../workspace.rs`
- `src/cli/admin/project_channels.rs` → `src/cli/admin/channels.rs`

### Key symbol renames
- Module `crate::project` → `crate::workspace`; `NoProject`→`NoWorkspace`, `project::project_root`→`workspace::workspace_dir`, `register_project`→`register_workspace`
- Table `project_roots` → `workspace_roots`; `ProjectRootBinding`→`WorkspaceBinding`; Store methods `project_root`→`workspace_path`, `project_root_binding`→`workspace_binding`, `list_project_root_bindings`→`list_workspace_bindings`, `upsert_project_root`→`upsert_workspace`
- Root-channel concept: `channel_project_root`→`root_channel_of`, `channel_resolve::project_root`→`root_channel`, `list_projects_read_model`→`list_root_channels`, `turn_context::project_root_h`→`root_channel_h`, fabric_context `ProjectRow`→`WorkspaceRow` (view field `project`→`workspace`), who_snapshot `OtherProjectSummary`→`OtherRootSummary` (+ fields → `root`/`root_display`/`other_roots`; `WhoRow.project`→`channel`)
- Probe validate `project_root` target/evidence → `workspace` (JSON `workspace_evidence`, check `workspace`, field `root_channel`)

### RPC method renames (wire skew accepted; dispatch, `daemon/blocking.rs`, handlers, CLI callers, and `daemon_integration` tests all updated)
- `project_add`→`channel_add_member`, `project_members`→`channel_members`, `project_remove`→`channel_remove_member`, `project_list`→`root_channels`, `project_edit` **deleted** (orphaned). JSON params/keys `"project"`→`"channel"`.

### CLI flags
- `--project`/`--all-projects` → `--root`/`--all-roots` (who, launch, channel list, agent assign, debug filter).

### Table migration (local state)
`src/state/schema/workspace_roots_migration.rs` (`ensure_renamed`, wired into both `initialize_file`/`initialize_memory` in `schema.rs` after DDL) copies legacy `project_roots` rows into `workspace_roots` and drops the old table; fresh/already-migrated DBs no-op. Covered by two unit tests. The on-disk map `~/.tenex-edge/projects.json` → `workspaces.json` with a one-time legacy read-fallback.

### Intentionally-left "project" (all verified legitimate)
- **NIP-29 wire**: I renamed the internal identifiers `project_tag`→`h_tag` and `project_from_tags`→`channel_from_tags` (purging wording) but did **not** touch any wire tag string (`"h"`, `"parent"`, etc.) or kinds — the protocol is unchanged.
- `src/workspace.rs` (2×) and `src/state/schema/workspace_roots_migration.rs` — the literal legacy `projects.json` / `project_roots` names, required for backward-compatible migration.
- `src/session.rs:5` and `src/cli/probe/tests.rs` (`*_action_projects_rpc_params`) — the English **verb** "projects" (data projection / maps-onto), not the noun.
- `src/cli/args.rs` and `src/cli/admin/args/tests.rs` — two tests that assert the already-removed `project` command / `--project` alias stays unavailable (negative guards).

One `daemon_integration` test (`pty_spawn_bootstraps_session_without_child_session_start_hook`) flaked once under full-suite load (relay membership timing) but passes in isolation and in the clean full run; leaked `nak serve --quiet` relays from prior interrupted runs were the cause and were cleaned up.
