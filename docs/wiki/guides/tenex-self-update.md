---
title: TENEX Self-Update Mechanism
slug: tenex-self-update
topic: architecture
summary: On macOS, when reinstalling a binary that gets SIGKILLed due to a stale code-signature, the fix is to remove the old binary and recopy it rather than overwritin
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-17
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:f80014e1-8264-4c3e-a8a6-a89718a6518a
---

# TENEX Self-Update Mechanism

## macOS In-Place Binary Update

On macOS, when reinstalling a binary that gets SIGKILLed due to a stale code-signature, the fix is to remove the old binary and recopy it rather than overwriting in place.

<!-- citations: [^95659-7] [^f8001-1] -->
