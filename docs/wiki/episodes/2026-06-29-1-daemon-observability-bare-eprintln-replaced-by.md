---
type: episode-card
date: 2026-06-29
session: 47f3cac2-1ad9-461c-8ac0-3ea341d0e962
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/47f3cac2-1ad9-461c-8ac0-3ea341d0e962.jsonl
salience: product
status: active
subjects:
  - daemon-logging
  - tracing-migration
  - cli-daemon-command
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 438-461
  - 517-544
  - 1215-1260
  - 1340-1459
captured_at: 2026-06-29T10:52:47Z
---

# Episode: Daemon observability: bare eprintln replaced by structured tracing with custom colored formatter

## Prior State

The daemon used bare `eprintln!` calls scattered across lifecycle.rs, demux.rs, engine_lifecycle.rs, session_start.rs, session_signer.rs, identity.rs, idle.rs, diagnostics.rs, tmux_rpc.rs, and relay_log.rs. There was no structured logging framework, no log-level policy, no colorized output, and no dual stdout+file logging. The `__daemon` subcommand was hidden from CLI help.

## Trigger

User explicitly requested comprehensive daemon logging: colorized stdout output plus daemon.log file, covering session routing, agent start reasons, ordinal creation reasons, relay events, and subscriptions. After initial implementation, user showed a screenshot and said 'this is very light on colors and on usefulness' — raw `[→relay]`/`[daemon]` eprintln noise was cluttering output, only level badges were colored, and status heartbeats fired every 30s at INFO level.

## Decision

Adopt `tracing` + `tracing-subscriber` as the daemon's logging framework with a custom `DaemonFormatter` (`src/logging.rs`). All `eprintln!` calls replaced by structured `tracing::*!` macros with a defined level policy: ERROR=unrecoverable, WARN=recoverable-unexpected, INFO=consequential-single, DEBUG=high-frequency/expected. Relay log entries (`relay_log.rs`) routed through `tracing::debug!` so they're suppressed by default. Status heartbeat (kind:30315) first-sight logs downgraded to debug. `tenex-edge daemon` exposed as a visible CLI subcommand (with `__daemon` alias for backward-compatible auto-spawner). Dual-output: ANSI colored stdout + plain-text daemon.log when foreground; single plain layer when detached.

## Consequences

- New `src/logging.rs` module with custom `FormatEvent` impl using owo-colors: level pills with colored backgrounds, bold white messages, dim field keys, bright_cyan field values
- `tracing` and `tracing-subscriber` are now direct Cargo.toml dependencies
- `RUST_LOG=tenex_edge=debug` is the operator's escape hatch for verbose output; default filter is `tenex_edge=info`
- Relay event lines (`[→relay]`, `[relay✗]`) no longer appear on foreground stdout at default log level — only in relay.log file and at debug level
- NIP-29 role decisions and status heartbeat first-sight logs suppressed by default
- `tenex-edge daemon` is now visible in `--help` output as a first-class subcommand
- Log level policy established as a durable convention for future daemon code: any new log point must choose ERROR/WARN/INFO/DEBUG per the defined semantics

## Open Tail

- Pre-existing unused-import warnings in server.rs and provider.rs remain unfixed (not introduced by this change)

## Evidence

- transcript lines 1-1
- transcript lines 438-461
- transcript lines 517-544
- transcript lines 1215-1260
- transcript lines 1340-1459

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-daemon-observability-bare-eprintln-replaced-by.json`](transcripts/2026-06-29-1-daemon-observability-bare-eprintln-replaced-by.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-daemon-observability-bare-eprintln-replaced-by.json`](transcripts/raw/2026-06-29-1-daemon-observability-bare-eprintln-replaced-by.json)
