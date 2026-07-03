---
title: Scoped Formatting
slug: scoped-formatting
topic: code-organization
summary: When a refactor runs `cargo fmt`, formatting churn on files unrelated to the change is reverted so the diff stays scoped.
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
  - session:4e6163df-c3cd-4d85-99ad-041cd0ca9701
---

# Scoped Formatting

## Scope Discipline

When a refactor runs `cargo fmt`, formatting churn on files unrelated to the change is reverted so the diff stays scoped. <!-- [^019f1-3c7ec] -->

Before merging a PR, the full `cargo test --lib` suite (382 tests) must pass, and `cargo clippy` and `cargo fmt` must be clean. <!-- [^4e616-19dcd] -->
