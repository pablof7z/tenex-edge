---
title: Module Visibility
slug: module-visibility
topic: code-organization
summary: Extracted module surfaces use narrow visibility (`pub(super)` or `pub(crate)`) rather than broad `pub` exposure; visibility is only widened when a consumer outs
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

# Module Visibility

## Extracted Module Visibility

Extracted module surfaces use narrow visibility (`pub(super)` or `pub(crate)`) rather than broad `pub` exposure; visibility is only widened when a consumer outside the module genuinely needs it. <!-- [^019f1-e9fbf] -->

Integration-test helper files under `tests/` live in a subdirectory and are loaded with explicit `#[path]` annotations so they do not become standalone test crates. <!-- [^019f1-eb33f] -->
