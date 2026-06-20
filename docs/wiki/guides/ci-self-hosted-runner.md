---
title: Self-Hosted CI Runner Saturation
slug: ci-self-hosted-runner
topic: ci-cd
summary: The repository uses a single self-hosted GitHub Actions runner (Pablos-MacBook-Pro-3-podcast) that processes one job at a time, causing saturation when multiple
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

# Self-Hosted CI Runner Saturation

## Self-Hosted Runner Configuration

The repository uses a single self-hosted GitHub Actions runner (Pablos-MacBook-Pro-3-podcast) that processes one job at a time, causing saturation when multiple agents push concurrently. <!-- [^74fce-4] -->
