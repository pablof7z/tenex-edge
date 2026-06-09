# Codex integration

Codex integrates through Codex lifecycle hooks. The hook dispatcher is
`te-hook.py`:

    integrations/codex/te-hook.py

Install the hooks by copying `config.template.toml` into either:

- `~/.codex/config.toml` for all Codex projects
- `<repo>/.codex/config.toml` for one trusted project

Replace `__HOOK__` with the absolute path to `integrations/codex/te-hook.py`.
Codex requires non-managed hooks to be reviewed and trusted in `/hooks`.

The hook mapping is:

- `SessionStart` -> `tenex-edge session-start --agent codex --session-id <codex-session-id> --cwd <cwd>`
- `UserPromptSubmit` -> `tenex-edge turn-start --session <codex-session-id> --transcript <transcript_path>` (marks the
  turn "working" so the engine starts its distillation timer), and inject pending mentions and `tenex-edge who` output
  as developer context
- `Stop` -> `tenex-edge turn-end --session <codex-session-id>` (marks the session idle when the turn finishes)

The adapter accepts Codex session identifiers under `session_id`, `sessionId`, `conversation_id`, `conversationId`,
`thread_id`, or `threadId`, then uses that value consistently as the tenex-edge session id. Codex hook payloads may also
carry a live `transcript_path` (a JSONL rollout file Codex keeps appending to during the turn), so the transcript path is
handed once at `turn-start` and the engine re-reads that same file as the turn progresses. Distillation is driven by the
turn lifecycle, not individual tool calls — there is no `PostToolUse` hook. Note that Codex hooks only fire in the
interactive TUI, not in `codex exec`.

For diagnostics, the adapter appends quiet status lines to `~/.tenex/edge/codex-hook.log`; set
`TENEX_EDGE_HOOK_LOG=/path/to/log` to override that location.

Codex does not currently document a `SessionEnd` hook. The dispatcher therefore
passes Codex's process id to `session-start --watch-pid` when it can identify the
Codex ancestor process; the tenex-edge liveness reaper stops presence when Codex
exits.
