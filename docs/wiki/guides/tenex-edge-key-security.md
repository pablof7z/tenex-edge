---
title: tenex-edge Key Security Incident
slug: tenex-edge-key-security
topic: security
summary: The owner key (09d48a1aâ¦) was leaked to Google during blind adb sign-in automation (typed into the emulator's search box) and must be rotated immediately
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-13
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
---

# tenex-edge Key Security Incident

## Key Rotation

The owner key (09d48a1a…) was leaked to Google during blind adb sign-in automation (typed into the emulator's search box) and must be rotated immediately. Additionally, the Ollama Cloud API key present in the tenex-off iOS scratch worktree (AppCoordinator.swift, commit 914cfb8) was preserved locally and not pushed to the remote, but should be treated as compromised and rotated as well.

<!-- citations: [^ab999-16] [^74fce-15] -->
