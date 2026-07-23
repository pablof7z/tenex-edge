# Codex integration

Codex integrates through Codex lifecycle hooks. Each hook calls the
`mosaico` binary directly:

    mosaico harness hook codex --type <hook-type>

Install the hooks by copying `config.template.toml` into either:

- `~/.codex/config.toml` for all Codex projects
- `<repo>/.codex/config.toml` for one trusted project

`mosaico` must be on PATH. Codex requires non-managed hooks to be reviewed
and trusted in `/hooks`.

The hook mapping is:

- `SessionStart` → `mosaico harness hook codex --type session-start`
- `UserPromptSubmit` → `mosaico harness hook codex --type user-prompt-submit`
  (marks the turn "working", injects pending chat messages and `mosaico who` output)
- `PostToolUse` → `mosaico harness hook codex --type post-tool-use`
  (read-only chat peek mid-turn; does not drain)
- `Stop` → `mosaico harness hook codex --type stop`
  (marks the session idle when the turn finishes)

The adapter reads the Codex session identifier from `session_id` and uses that
value consistently as the mosaico session id.

Codex hooks only fire in the interactive TUI, not in `codex exec`.

Codex does not currently document a `SessionEnd` hook. The binary detects the
Codex ancestor process via `pid_search` and passes it to `session-start
--watch-pid`; the liveness reaper stops presence when Codex exits.
