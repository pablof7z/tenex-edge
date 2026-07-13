---
title: File Size Limits
slug: file-size-limits
topic: code-organization
summary: Hand-authored source and documentation files should stay under 300 lines of code (soft limit), with 500 lines as a hard ceiling enforced by the CI `fmt · lint ·
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-13
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
  - session:019f12ce-2569-72e0-b959-6d87d5daec5d
  - session:4e6163df-c3cd-4d85-99ad-041cd0ca9701
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# File Size Limits

## Hand-Authored File Size Limits

Hand-authored source and documentation files should stay under 300 lines of code (soft limit), with 500 lines as a hard ceiling enforced by the CI `fmt · lint · loc · test` job's LOC ratchet check. When a code file crosses the 500-LOC hard limit, it is refactored by splitting responsibilities along domain boundaries (e.g., `channels_rpc/archive.rs` and `channels_rpc/list.rs`), not by moving arbitrary chunks to a sibling file. Inline tests that inflate a source file's LOC belong in a nested test module so the implementation stays under the soft target.

<!-- citations: [^019f1-85a20] [^019f1-f4e29] [^4e616-0635a] [^019f5-6b718] -->
## Exemptions

Generated, vendored, lockfile, binary, and benchmark-output artifacts are exempt from the LOC ceiling, but their producers must be kept small and documented.

<!-- citations: [^019f1-37c47] [^019f1-53558] [^019f5-6f214] -->
## Local Enforcement Gates

Local gates include `scripts/check_loc.sh` (LOC enforcement) and `cargo test --lib` (unit tests). The dirty worktree state fails `scripts/check_loc.sh` with three >500 LOC files and has one existing failing assertion in `cargo test --lib`.

The LOC ratchet also tracks a soft-limit drift threshold: files already over 300 lines are flagged if they drift more than a pre-existing baseline (e.g., 460 lines for `channels_rpc.rs`) during a change. However, the soft-limit drift check only triggers when `origin/master` is resolvable as a local ref, which GitHub's default shallow PR checkout (fetch-depth 1) does not provide, causing the check to no-op in real CI.

<!-- citations: [^019f1-bd714] [^4e616-1b4fe] -->
