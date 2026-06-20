---
title: iOS AppStateStore Dictionary Safety
slug: ios-app-state-store
topic: ios-app
summary: "AppStateStore.applyDownloadOverlay uses Dictionary(_:uniquingKeysWith:) instead of Dictionary(uniqueKeysWithValues:) for its active-downloads overlay, preventin"
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

# iOS AppStateStore Dictionary Safety

## Active-Downloads Overlay

AppStateStore.applyDownloadOverlay uses Dictionary(_:uniquingKeysWith:) instead of Dictionary(uniqueKeysWithValues:) for its active-downloads overlay, preventing a fatal error when duplicate keys appear from the kernel. <!-- [^74fce-11] -->
