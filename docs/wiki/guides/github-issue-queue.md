---
title: GitHub Issue Queue
slug: github-issue-queue
topic: repo-discipline
summary: "The repository has exactly one canonical tactical queue: GitHub Issues (`gh issue list`)"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-03
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
  - session:fea5307b-d9a0-46fe-977c-408e5e0e0ff4
---

# GitHub Issue Queue

## Canonical Queue

The repository has exactly one canonical tactical queue: GitHub Issues (`gh issue list`). Every active violation, pending user decision, queued feature, post-v1 item, staged fix, and follow-up belongs in an open GitHub issue. The canonical surfaces for planning are GitHub Issues plus durable docs. Open GitHub epics track CLI surface, store/read-model ownership, daemon service extraction, provider/fabric ownership, tooling, docs drift, and quality gates. <!-- [^019f1-5b3ac] -->

## Forbidden Tracking Patterns

Scattered notes, ad-hoc `TODO.md` / `NOTES.md` / `ROADMAP.md` / `PLAN-foo.md` files, parallel planning docs, and inline `// TODO:` annotations used as a substitute for tracking are forbidden. State is not duplicated across files; a violation or feature tracked in GitHub Issues is not also restated in a separate plan file. When in doubt, fewer planning files are preferred; if a new planning file feels necessary, the justification must explain why it cannot be a GitHub issue plus a durable doc. <!-- [^019f1-97693] -->

## Workflow

GitHub search is the backlog view; `gh issue list --state open` shows the queue, the issue is closed or updated in the PR that touches it, and lasting conclusions move into durable docs. Bugs or gaps found during mention-testing are filed as GitHub issues, ideally with screenshots of the observed behavior.

<!-- citations: [^019f1-8d5dd] [^fea53-03268] -->
