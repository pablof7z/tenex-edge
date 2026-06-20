---
type: episode-card
date: 2026-06-16
session: 9337d29e-ac62-417c-8e99-0cc22cbbfad3
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9337d29e-ac62-417c-8e99-0cc22cbbfad3.jsonl
salience: architecture
status: active
subjects:
  - opencode-integration
  - turn-context-injection
  - single-source-of-truth
supersedes: []
related_claims: []
source_lines:
  - 210-272
  - 428-446
captured_at: 2026-06-18T00:43:31Z
---

# Episode: opencode plugin becomes a dumb pipe — inject hook stdout instead of rebuilding context in TS

## Prior State

The opencode integration duplicated all context-building that the Rust hook path already assembled. It called `runHook("user-prompt-submit")` but discarded stdout, then rebuilt the self-identity line, inbox, and peer roster in TypeScript via bespoke `selfLine`, `tenex-edge inbox`, and `tenex-edge who` shell-outs. Mid-turn refreshes also used the destructive `inbox`/`who` CLI calls on every model invocation instead of the non-destructive peek path.

## Trigger

User questioned why the opencode integration must duplicate all the same context-assembly code instead of just consuming what the hook already prints to stdout. Investigation confirmed `runHook` already captures stdout — opencode was simply throwing it away.

## Decision

Refactored opencode to inject hook stdout verbatim: new user messages use `runHook("user-prompt-submit")` stdout (the full turn-start context: self-line + drained inbox + chat + presence), and mid-turn model invocations use `runHook("post-tool-use")` stdout (the non-destructive `turn_check` peek path). Deleted all bespoke TS context assembly: `selfLine`, `hinted`, `run()`, `stripAnsi()`, `SHORT_CODE`, and the `inbox`/`who` shell-outs.

## Consequences

- Self-identity line, inbox, chat, and presence now have a single source of truth in Rust `turn.rs` for all three harnesses (Claude Code, Codex, opencode)
- opencode now gets the proper drain-once / peek-mid-turn split instead of re-listing inbox+who on every model invocation
- Net −25 lines; deleted 64 lines of duplication, added 39 lines of pipe-through logic
- opencode agents now see the unified `tenex-edge inbox send --to` phrasing instead of the old opencode-specific `send-message` wording

## Open Tail

- If the parallel session-state rearchitecture (session `8c22fb`) changes what `user-prompt-submit`/`post-tool-use` emit on stdout, this plugin will automatically track it — but verify no semantic contract breaks

## Evidence

- transcript lines 210-272
- transcript lines 428-446

