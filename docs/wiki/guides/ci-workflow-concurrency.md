---
title: CI Workflow Concurrency
slug: ci-workflow-concurrency
topic: ci-cd
summary: "The Test workflow uses per-branch/per-PR concurrency with cancel-in-progress: true, so a newer push cancels the older in-flight run for that ref instead of stac"
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

# CI Workflow Concurrency

## Concurrency

The Test workflow uses per-branch/per-PR concurrency with cancel-in-progress: true, so a newer push cancels the older in-flight run for that ref instead of stacking runs. <!-- [^74fce-8] -->
