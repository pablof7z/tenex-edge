---
title: Code Size Limits
slug: code-size-limits
topic: code-standards
summary: All code files must remain under 500 LOC (hard limit)
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-10
updated: 2026-06-10
verified: 2026-06-10
compiled-from: conversation
sources:
  - session:rollout-2026-06-10T22-36-00-019eb308-d484-77d2-a8ee-03f5a676ed99
---

# Code Size Limits

## Code Size Limits

All code files must remain under 500 LOC (hard limit). Code files should be kept under 300 LOC (soft limit), with extraction considered by responsibility boundaries. Over-limit files must be refactored by splitting along responsibility and domain boundaries rather than arbitrary line cuts. <!-- [^rollo-27] -->

Rust integration-test helper files under `tests/` must live in a subdirectory and use `#[path]` rather than existing as standalone test crates. <!-- [^rollo-28] -->

The NIP-29 probe test support helpers (e.g. create-group retry loop) must be extracted into a separate support module to keep the probe scenario under the soft target. <!-- [^rollo-29] -->
