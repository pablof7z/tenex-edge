---
type: episode-card
date: 2026-06-13
session: 74fce09f-02b4-496f-a5e1-52d19ef9fbcd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/74fce09f-02b4-496f-a5e1-52d19ef9fbcd.jsonl
salience: architecture
status: active
subjects:
  - ci-testflight-trigger
  - ci-release-gate
  - deployment-signal
supersedes: []
related_claims: []
source_lines:
  - 2786-3006
captured_at: 2026-06-18T00:16:14Z
---

# Episode: TestFlight deploy now gates on version bump (not every push) + unit-only release criterion

## Prior State

Every push to main triggered a full TestFlight build+test+deploy cycle on the single self-hosted runner, including the simulator-flaky UI test suite.

## Trigger

User directive: 'deploy to testflight only when we increase version number instead of all the time.' Runner saturation (10+ queued jobs, 60+ min waits) made the old trigger unsustainable.

## Decision

TestFlight workflow now only runs when CFBundleShortVersionString in Info.plist changes (or on manual dispatch). A free-cloud gate job checks the version diff before spending the self-hosted runner. Deploy also gates on unit tests only (SKIP_UI_TESTS=1), decoupling the release from simulator-flaky UI tests.

## Consequences

- Routine pushes (features, fixes, wiki-autodocs) no longer consume any self-hosted runner time for deploys
- CFBundleShortVersionString is now the source-of-truth deployment signal
- UI test failures (jetsam kills, known kernel bugs) no longer block TestFlight uploads
- The full UI suite still runs on every PR via the separate Test workflow for coverage

## Open Tail

- Cut a 1.0.1 release to trigger the first gated deploy through the now-unblocked pipeline

## Evidence

- transcript lines 2786-3006

