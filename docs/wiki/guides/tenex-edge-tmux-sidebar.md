---
title: tenex-edge Tmux Sidebar
slug: tenex-edge-tmux-sidebar
topic: tenex-edge
summary: The `[no tmux]` tag has been removed from non-attachable live rows, and help text updated to `[âµ] attach/spawn`
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-15
updated: 2026-06-17
verified: 2026-06-15
compiled-from: conversation
sources:
  - session:9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf
  - session:a88513d3-754f-4369-b440-72c8d29331e2
  - session:rollout-2026-06-16T14-02-11-019ed018-926e-7c40-bf14-796efbec0b7a
  - session:rollout-2026-06-17T11-06-26-019ed49e-0783-7c50-a1db-0850a653f66c
  - session:rollout-2026-06-17T11-22-24-019ed4ac-a308-7250-b5ec-d95c8d18de3e
  - session:rollout-2026-06-17T12-04-57-019ed4d3-96b5-7b73-b076-4969a3d16afa
  - session:ses_1308a0757ffe8RvTvZs19Cq60r
  - session:ses_13041c14dffeZTVP5diY3jtQIQ
  - session:ses_130406996ffeA11HwriiOojv2k
---

# tenex-edge Tmux Sidebar

## Session List UI Details

The `[no tmux]` tag has been removed from non-attachable live rows, and help text updated to `[↵] attach/spawn`. The recent-projects visibility threshold is `TWELVE_HOURS` (changed from `SEVEN_DAYS`), and live project tabs are sorted by session count descending. <!-- [^9bab9-7] -->


The tenex-edge tmux TUI has no 'All' view; only per-project views are available. The default tab is the project corresponding to the current directory the command is run from. The tab bar starts directly with project names (no `[All]` tab), and navigation boundaries treat `tab_idx > 0` as the left limit and `tab_idx + 1 < pt.visible.len()` as the right limit. <!-- [^ses_1-39] -->
## Data Model & Helpers

The `LiveRow` struct includes a `project: String` field so the session list can filter sessions by project. The `current_tmux_session()` helper resolves the current session name by running `tmux display-message -p '#{client_session}'`. The old `tenex-edge tmux spawn` CLI subcommand is removed (unrecognized), while the underlying daemon RPC (`tmux_spawn`) and the TUI spawn path remain intact. The `WhoRow` struct in `who.rs` includes an `unread: usize` field, populated by `store.count_unread_inbox()` for local sessions and set to 0 for peer sessions. The `LiveRow` struct in `tmux_cli.rs` includes an `unread: usize` field, parsed from the `who` RPC JSON response. The `TmuxAction::Sidebar` enum variant is removed from the CLI. The `--popup` flag and popup field are removed from the Tmux CLI command.

The `tab_project` function returns `tabs.get(tab_idx)` directly, with no special index-0 case returning `None` for an All view. The `filter_live` and `filter_resumable` functions take `&str` (not `Option<&str>`) and always filter by project. The `render_scrolled_body` function takes `project_filter` as `&str` instead of `Option<&str>`. The `render_main` function derives `project_filter` from `tabs.get(tab_idx)` and returns early if no tabs are present. The `update_tabs_after_refresh` function uses `tab_idx` directly instead of `tab_idx - 1` and preserves the selected project tab with no special All case. In search mode, selecting a project sets `tab_idx = idx` (not `idx + 1`). The `live_row_line` and `resume_row_line` functions have no `project_filter` parameter and always display `slug@host` and `slug` respectively. <!-- [^ses_1-40] -->

<!-- citations: [^ses_1-24] [^9bab9-8] [^a8851-2] [^ses_1-37] -->
## Message Injection in Live Sessions

Incoming message listening is handled via `wait-for-mention` (blocks until a mention arrives) or `inbox` (reads and drains pending mentions), not by the `tmux` TUI itself. The tmux daemon delivers incoming messages to sessions not by listening, but by actively injecting a doorbell nudge into idle tmux panes via send-keys whenever a mention arrives, prompting the agent to run `tenex-edge inbox`. The manual `tenex-edge tmux send` RPC uses the same real-message injection path as the live tmux receive path. The tmux injection path renders only explicit chat mentions and direct messages, not all unread rows. When a session-start hook records or refreshes a tmux pane endpoint, the daemon immediately rings pending-message delivery so that unread inbox/chat rows are pasted into the pane even when tenex-edge tmux is not running. Tmux delivery skips working sessions, stale panes, and debounce windows.

Sessions with unread mentions display a yellow `◉N` indicator (where N is the unread count) in the tmux TUI session list and in `tenex-edge who` text output. <!-- [^ses_1-23] -->

<!-- citations: [^rollo-71] [^rollo-116] [^ses_1-22] -->

## Session Attach & Detach Behavior

When a user detaches from an agent's tmux pane (e.g. Ctrl-b d), they return to the tenex-edge tmux TUI session. <!-- [^ses_1-38] -->
