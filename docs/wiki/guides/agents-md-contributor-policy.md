---
title: AGENTS.md Contributor Policy
slug: agents-md-contributor-policy
topic: engineering-standards
summary: AGENTS.md must include a contributor policy that sets a soft limit of 300 LOC and a hard limit of 500 LOC for code files
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

# AGENTS.md Contributor Policy

## Code Size Limits

AGENTS.md must include a contributor policy that sets a soft limit of 300 LOC and a hard limit of 500 LOC for code files. Code files over the 500 LOC hard limit must be refactored by splitting responsibilities along domain boundaries rather than by moving arbitrary chunks. <!-- [^rollo-33] -->

Inline tests inflating a source file's LOC should be moved into a nested test module (e.g., `tests.rs`) to keep the implementation under the soft target. <!-- [^rollo-34] -->

The ignored live NIP-29 probe scenario must be split into separate support/helper modules for reusable operations (e.g., create-group retry loops, Q1 readback) to stay under the 300 LOC soft target. <!-- [^rollo-35] -->

## Module Visibility

Extracted module surfaces should use narrow visibility (e.g., `pub(super)` or `pub(crate)`) rather than broad public exposure. <!-- [^rollo-36] -->

## Integration Test Helpers

Integration test helper files under the `tests/` directory must live in a subdirectory and be loaded with explicit `#[path]` annotations to prevent them from becoming standalone test crates. <!-- [^rollo-37] -->

## Refactor Diff Hygiene

Formatting churn on unrelated files introduced by `cargo fmt` must be reverted so that the refactor diff remains scoped. <!-- [^rollo-38] -->
