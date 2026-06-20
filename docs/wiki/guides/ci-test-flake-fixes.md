---
title: CI Test Flake Fixes
slug: ci-test-flake-fixes
topic: ci-cd
summary: The run_tests.sh script includes a pre-build chmod -R u+w on the secp256k1 DerivedData plugin cache, self-healing the read-only flake caused by interrupted buil
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

# CI Test Flake Fixes

## Test Infrastructure Self-Healing

The run_tests.sh script includes a pre-build chmod -R u+w on the secp256k1 DerivedData plugin cache, self-healing the read-only flake caused by interrupted builds. <!-- [^74fce-5] -->
