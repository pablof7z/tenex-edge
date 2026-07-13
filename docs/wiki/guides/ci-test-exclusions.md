---
title: CI Test Exclusions
slug: ci-test-exclusions
topic: repo-discipline
summary: Daemon integration tests are excluded from CI
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-10
updated: 2026-07-13
verified: 2026-07-10
compiled-from: conversation
sources:
  - session:af454e46-7c4f-4182-ab2b-ebc50b1eb9ad
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# CI Test Exclusions

## CI Test Exclusions

Daemon integration tests are excluded from CI. Integration-test helper files under `tests/` must live in a subdirectory and be loaded with explicit `#[path]` annotations so they do not become standalone test crates.

<!-- citations: [^af454-63cf7] [^019f5-4c80b] -->
