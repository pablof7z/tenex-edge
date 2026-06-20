---
type: episode-card
date: 2026-06-15
session: 9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9bab94a2-f76f-4eda-ae41-8a6ec29ce7cf.jsonl
salience: product
status: active
subjects:
  - tenex-edge-tmux-sidebar
  - tenex-edge-tmux-popup
  - session-switching
supersedes: []
related_claims: []
source_lines:
  - 85-113
  - 126-189
  - 551-614
  - 616-705
  - 788-897
  - 915-1155
captured_at: 2026-06-18T00:33:35Z
---

# Episode: Always-visible session sidebar + popup quick-switcher for tmux sessions

## Prior State

No way to switch between sessions while attached to a tmux session. User had to detach back to the TUI list, then re-attach to another session. The only option was tmux's built-in Ctrl-b s session chooser with uninformative names.

## Trigger

User asked: 'would we be able to show a sidebar next to the main tab with the other sessions in the project so the user can quickly switch?' Three options were presented (A: persistent split-pane sidebar, B: popup on keypress, C: tmux built-in). User initially tested option B, then directed: 'build the sidebar on the master so that its always visible'.

## Decision

Implemented two complementary session-switching mechanisms: (1) Always-visible 40-col fixed-width left sidebar pane injected into each agent's tmux window on attach via `ensure_sidebar()` running `tenex-edge tmux sidebar --session <id>`. Current session highlighted in cyan bold with ►, ↑/↓ to navigate, Enter to switch (injects sidebar into target then `switch-client`). (2) M-t popup quick-switcher via `display-popup -E` running `tenex-edge tmux --popup`, which does `switch-client` on the underlying client then exits (closing the popup) instead of attaching inline. Sidebar uses ratatui rendering; `client-resized` hook re-pins width on resize.

## Consequences

- Sidebar auto-injected on every attach via `ensure_sidebar()` — idempotent, checks for existing sidebar pane before splitting
- M-s focuses sidebar, M-a focuses agent pane — bound via `bind_sidebar_keys()` on every attach
- Popup mode uses `--popup` CLI flag to branch: switch-client + exit instead of inline attach, preventing sessions from being trapped inside the popup overlay
- Sidebar fixed at 40 cols via `resize-pane -x 40` in a `client-resized` hook, preventing proportional rescaling on window resize
- Both sidebar and popup launch `tenex-edge` by bare name, requiring it on $PATH (not yet switched to `std::env::current_exe()`)

## Open Tail

- Both sidebar and popup resolve `tenex-edge` by bare name — needs `std::env::current_exe()` for dev builds
- Popup is relative-sized (80%×80%); user may want fixed sizing too

## Evidence

- transcript lines 85-113
- transcript lines 126-189
- transcript lines 551-614
- transcript lines 616-705
- transcript lines 788-897
- transcript lines 915-1155

