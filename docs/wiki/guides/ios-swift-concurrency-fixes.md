---
title: iOS Swift Concurrency Fixes
slug: ios-swift-concurrency-fixes
topic: ios-app
summary: The test method calling `nostrConversationFromDTO` was annotated `@MainActor` to fix a compile error from calling a main-actor-isolated static method in a synch
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

# iOS Swift Concurrency Fixes

## Swift Concurrency Fixes

The test method calling `nostrConversationFromDTO` was annotated `@MainActor` to fix a compile error from calling a main-actor-isolated static method in a synchronous nonisolated context. <!-- [^74fce-12] -->
