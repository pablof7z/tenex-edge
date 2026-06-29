---
title: Planning vs Durable Docs
slug: planning-vs-durable-docs
topic: repo-discipline
summary: Plans are not durable understanding and must not survive as reference documentation after they have been implemented, executed, or invalidated
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
---

# Planning vs Durable Docs

## Plans Are Not Durable Documentation

Plans are not durable understanding and must not survive as reference documentation after they have been implemented, executed, or invalidated. When a plan completes, it is removed or collapsed to the smallest live follow-up; lasting knowledge learned from the work belongs in durable documentation instead. An implemented plan is no longer a source of truth; the issue is closed or the temporal detail is deleted, and durable lessons are preserved in the doc that owns that concept. <!-- [^019f1-1e264] -->

## TODO Comments

A `// TODO:` comment is not a plan. If it represents work to be done it belongs in a GitHub issue, and if it represents a known limitation or durable decision it belongs in the architecture/design doc or wiki article that owns the concept. <!-- [^019f1-eb16d] -->

## Review Dumps and Post-Merge Notes

AI/codex review dumps and post-merge review notes must not be committed. Actionable findings are promoted into a GitHub issue or a durable doc, then the review is discarded. <!-- [^019f1-11045] -->
