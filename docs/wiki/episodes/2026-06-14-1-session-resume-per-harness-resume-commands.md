---
type: episode-card
date: 2026-06-14
session: 9f7f245f-0fad-4211-a86b-95ea3cbb532e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9f7f245f-0fad-4211-a86b-95ea3cbb532e.jsonl
salience: architecture
status: superseded
subjects:
  - session-resume
  - spawn-def-resume-spec
  - opencode-resume-id
supersedes:
  - 2026-06-14-1-resume-spawn-path-for-dead-agent
related_claims: []
source_lines:
  - 41-72
  - 74-77
  - 110-178
  - 492-524
captured_at: 2026-06-14T11:24:52Z
---

# Episode: Session resume: per-harness resume commands and separate resume_id storage

## Prior State

tenex-edge could only attach to live tmux panes; dead sessions were unrecoverable. Sessions stored a single session_id. For claude/codex this was also the harness-native resume token, but for opencode the stored te-* synthetic ID was not what opencode --session expects. SpawnDef only knew cold-start commands. The opencode plugin had the native ses_* token locally but never forwarded it to the hook.

## Trigger

User asked whether tenex-edge can resume sessions initiated outside tmux. Investigation revealed: (1) claude/codex already store the native resume token; (2) opencode's ses_* token is available in the plugin but not forwarded; (3) each harness has a different resume-command shape (claude: --resume flag, codex: resume subcommand, opencode: --session flag).

## Decision

Add a dedicated resume_id column on sessions (distinct from identity session_id); forward opencode's native ses_* ID through the hook as resume_id; extend SpawnDef with per-harness resume specs (flag vs subcommand, token name); build a resume spawn path that constructs the correct argv per harness from the agent's configured launch command + resume flag/subcommand + resume_id. Resume is local-only — keyed off session_endpoints (session→pane) with a liveness check: live pane → attach, dead pane → resume in new window.

## Consequences

- claude/codex sessions have resume_id == session_id (no migration needed); opencode sessions gain a new resume_id holding the ses_* token
- SpawnDef gains a resume_spec field defining command shape (append-flag vs subcommand) and token name (--resume, --session, resume)
- Resume command reuses agent.json's command field (e.g. claude --dangerously-skip-permissions) with the resume token appended per shape
- TUI/CLI needs a new 'resume' action distinct from 'attach', conditioned on pane liveness
- opencode plugin now forwards ocSessionID in its hook payload

## Open Tail

- Actual TUI/CLI resume action entry point still to be wired
- Pane liveness check to decide attach-vs-resume not yet implemented
- Whether resumed sessions reuse the original session row/short-code or create a new one pointing back (affects inbox routing and fabric listing)

## Evidence

- transcript lines 41-72
- transcript lines 74-77
- transcript lines 110-178
- transcript lines 492-524

