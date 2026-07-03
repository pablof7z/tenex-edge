---
title: Container State Cleanup
slug: container-state-cleanup
topic: repo-discipline
summary: Stale `.container-state` profiles from prior lab sessions accumulate Rust build caches (~2.4GB each) and are gitignored disposable state that can block new cont
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:fea5307b-d9a0-46fe-977c-408e5e0e0ff4
---

# Container State Cleanup

## Overview

Stale `.container-state` profiles from prior lab sessions accumulate Rust build caches (~2.4GB each) and are gitignored disposable state that can block new container builds when the disk fills. <!-- [^fea53-a160b] -->
