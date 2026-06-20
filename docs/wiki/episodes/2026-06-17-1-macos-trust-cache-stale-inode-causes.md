---
type: episode-card
date: 2026-06-17
session: f80014e1-8264-4c3e-a8a6-a89718a6518a
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/f80014e1-8264-4c3e-a8a6-a89718a6518a.jsonl
salience: root-cause
status: active
subjects:
  - macos-trust-cache
  - binary-install
  - sigkill-137
supersedes: []
related_claims: []
source_lines:
  - 425-513
captured_at: 2026-06-18T00:47:52Z
---

# Episode: macOS trust-cache stale inode causes SIGKILL on identical binary

## Prior State

tenex-edge installed at ~/.local/bin/ was being SIGKILLed (exit 137) on every invocation, despite the binary being byte-for-byte identical to the freshly-built target/debug/tenex-edge that ran fine.

## Trigger

Diagnosis showed the two files were byte-identical (same MD5) but macOS had a stale code-signature / trust-cache entry bound to the old inode at ~/.local/bin/tenex-edge, causing the kernel to kill the process at launch.

## Decision

Remove the old binary and recopy (rm + cp) to force a fresh inode, clearing the stale trust-cache entry. The binary then ran correctly.

## Consequences

- Future binary updates must rm the old path before cp'ing, not just cp over it, to avoid stale trust-cache SIGKILLs on macOS.
- Alternative mitigations include cp -R or sudo /usr/bin/killall -u $USER amfid to flush the trust cache.

## Open Tail

- The install/rebuild workflow should be hardened (e.g., a Makefile install target that does rm-before-cp) to prevent recurrence.

## Evidence

- transcript lines 425-513

