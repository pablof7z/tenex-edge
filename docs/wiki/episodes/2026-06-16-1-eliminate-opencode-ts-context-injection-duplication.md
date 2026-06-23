---
type: episode-card
date: 2026-06-16
session: 9337d29e-ac62-417c-8e99-0cc22cbbfad3
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/9337d29e-ac62-417c-8e99-0cc22cbbfad3.jsonl
salience: architecture
status: superseded
subjects:
  - opencode-integration
  - turn-context-injection
  - session-identity
supersedes: []
related_claims: []
source_lines:
  - 1-271
captured_at: 2026-06-16T10:53:56Z
---

# Episode: Eliminate opencode TS context-injection duplication by consuming Rust hook stdout

## Prior State

opencode's TypeScript integration discards the stdout from `runHook("user-prompt-submit")` and independently rebuilds the same context blocks (self-line, inbox, who) in TS, creating a parallel injection path that duplicates what the Rust `turn_start` already assembles. This forced the recent session-identity commit to touch both Rust and TS separately.

## Trigger

User noticed session-identity injection was added in two places and asked why opencode can't just read what the tenex-edge hook prints instead of duplicating all the same code in TS.

## Decision

opencode should consume the stdout from `runHook("user-prompt-submit")` directly for first-turn context injection, and map its repeated mid-turn `transform` calls to the existing `turn_check` (peek) path instead of shelling out to `tenex-edge inbox` / `who` separately. This makes opencode a dumb pipe like the other harnesses and eliminates the TS parallel path entirely.

## Consequences

- Single source of truth for all context blocks lives in Rust `turn.rs`; no more TS selfLine / inbox / who rebuilds
- The original justification for the parallel path — that `turn_start` drains inbox destructively — is moot because `turn_check` already provides the non-destructive peek semantics opencode needs mid-turn
- Future changes to context injection (e.g. adding new block types) only need to touch the Rust path
- Net code deletion: TS selfLine, `tenex-edge inbox` shell-out, `tenex-edge who` shell-out all removed

## Open Tail

- Implementation not yet started — awaiting user go-ahead
- Need to verify opencode's `experimental.chat.messages.transform` can reliably inject raw stdout text into message parts without losing formatting

## Evidence

- transcript lines 1-271

