---
type: episode-card
date: 2026-06-13
session: 74fce09f-02b4-496f-a5e1-52d19ef9fbcd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/74fce09f-02b4-496f-a5e1-52d19ef9fbcd.jsonl
salience: architecture
status: active
subjects:
  - ci-test-concurrency
supersedes: []
related_claims: []
source_lines:
  - 2852-2858
  - 2940-2957
captured_at: 2026-06-18T00:16:14Z
---

# Episode: Test workflow: cancel-in-progress per branch/PR

## Prior State

Test workflow runs stacked 10+ deep on the single self-hosted runner because every push (main + PR) created a new in-flight job; older runs were never cancelled.

## Trigger

Runner saturation was the root cause of CI deadlock. User asked for a recommendation on what to do about the Test workflow.

## Decision

Added concurrency group per branch/PR with cancel-in-progress: true to the Test workflow. Newer pushes cancel older in-flight iOS test runs for the same ref.

## Consequences

- Queue no longer grows unboundedly — only the latest commit per ref occupies the runner
- Safe because run_tests.sh now self-heals the secp256k1 DerivedData permission flake (PR #473), so cancelled builds don't corrupt shared state
- The non-self-hosted Test jobs (diff-check, android, codegen-drift, etc.) are unaffected as they run on free cloud runners

## Open Tail

*(none)*

## Evidence

- transcript lines 2852-2858
- transcript lines 2940-2957

