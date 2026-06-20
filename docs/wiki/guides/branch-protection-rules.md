---
title: Branch Protection Rules
slug: branch-protection-rules
topic: ci-cd
summary: "Branch protection on `main` requires six cloud-based status checks to pass before a PR can merge: Rust workspace build gate, Swift bridge codegen drift gate, An"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-13
updated: 2026-06-13
verified: 2026-06-13
compiled-from: conversation
sources:
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
---

# Branch Protection Rules

## Branch Protection Rules

Branch protection on `main` requires six cloud-based status checks to pass before a PR can merge: Rust workspace build gate, Swift bridge codegen drift gate, Android Kotlin compile + unit tests, Android cross-compile check, Headless e2e kernel proofs, and Git diff hygiene. A PR is required to merge, but zero approvals are needed, allowing agents to self-merge once all checks are green. Strict up-to-date status checks are not enforced (strict: false), preventing forced re-runs on every intervening merge. Admin restrictions are not enforced (enforce_admins: false), so the repo admin can bypass or override protection in a pinch. The self-hosted iOS 'Build and Test' check is deliberately excluded from required branch protection checks to avoid the contended runner blocking all merges. <!-- [^74fce-3] -->
