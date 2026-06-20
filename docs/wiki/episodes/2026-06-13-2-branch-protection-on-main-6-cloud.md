---
type: episode-card
date: 2026-06-13
session: 74fce09f-02b4-496f-a5e1-52d19ef9fbcd
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/74fce09f-02b4-496f-a5e1-52d19ef9fbcd.jsonl
salience: architecture
status: active
subjects:
  - branch-protection
  - ci-merge-policy
supersedes: []
related_claims: []
source_lines:
  - 3009-3087
captured_at: 2026-06-18T00:16:14Z
---

# Episode: Branch protection on main: 6 cloud checks required before merge

## Prior State

No branch protection on main; fleet agents merged directly with zero CI validation. Compile-broken commits landed repeatedly (LLMProviderTests, stale selectors), breaking main on a rolling basis.

## Trigger

Fleet merged un-CI'd PRs (#435–#473+) that repeatedly broke main. Assistant recommended requiring cloud checks; user approved with 'yes!'.

## Decision

GitHub branch protection on main requires 6 cloud-based status checks to pass before merge: Rust workspace build, Swift codegen drift, Android Kotlin, Android cross-compile, Headless e2e, Git diff hygiene. PRs required but 0 approvals needed; strict: false (no forced re-runs on merge); admin bypass allowed.

## Consequences

- Compile-broken commits can no longer reach main
- Fleet agents must go through PRs instead of pushing directly
- Cloud checks act as a natural throttle (parallel, ~2 min) without blocking on the contended self-hosted runner
- Admins can still override/bypass in emergencies

## Open Tail

- Other active agents will now see merges blocked until their PRs go green — may need workflow adjustments on their side

## Evidence

- transcript lines 3009-3087

