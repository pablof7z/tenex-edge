---
type: episode-card
date: 2026-06-26
session: 2e5bb74a-36e4-4dda-b695-24e9ca611411
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/2e5bb74a-36e4-4dda-b695-24e9ca611411.jsonl
salience: root-cause
status: active
subjects:
  - distill
  - claude-cli
  - system-prompt
supersedes: []
related_claims: []
source_lines:
  - 1-9
  - 40-46
  - 86-91
  - 93-102
  - 104-114
captured_at: 2026-06-26T19:19:12Z
---

# Episode: Distillation system prompt delivery: temp file → inline argument

## Prior State

The distill.rs daemon wrote the session system prompt to a temp file and invoked `claude -p` with `--system-prompt-file <path>`. Cross-environment temp directory differences between daemon and shell caused repeated ENOENT errors, forcing repeated fallback to nudge-to-keep.

## Trigger

User reported 'all claude based distillation is buggy' with repeated ENOENT Bun errors. Root-cause analysis revealed the daemon's $TMPDIR differs from the shell's, breaking file path assumptions.

## Decision

Eliminate the temp file entirely. Pass the system prompt directly to claude CLI using the `--system-prompt` argument instead of `--system-prompt-file`.

## Consequences

- Removes temp directory path dependency—no ENOENT failure mode at all
- Distillation now succeeds reliably across daemon environments without fallback
- Simpler implementation: one fewer file I/O cycle per distillation
- Cross-environment robustness improved (no assumptions about shared temp dirs)

## Open Tail

- Daemon process must be restarted to pick up the updated distill.rs

## Evidence

- transcript lines 1-9
- transcript lines 40-46
- transcript lines 86-91
- transcript lines 93-102
- transcript lines 104-114

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-26-1-distillation-system-prompt-delivery-temp-file.json`](transcripts/2026-06-26-1-distillation-system-prompt-delivery-temp-file.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-26-1-distillation-system-prompt-delivery-temp-file.json`](transcripts/raw/2026-06-26-1-distillation-system-prompt-delivery-temp-file.json)
