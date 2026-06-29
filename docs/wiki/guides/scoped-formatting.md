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
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
---

# Scoped Formatting

## Scope Discipline

When a refactor runs `cargo fmt`, formatting churn on files unrelated to the change is reverted so the diff stays scoped. <!-- [^019f1-3c7ec] -->
