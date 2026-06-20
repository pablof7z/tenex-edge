---
title: TENEX Role Separation
slug: tenex-role-separation
topic: architecture
summary: "TENEX enforces three strictly separated roles: Subscribe (relay), Orchestrate (runtime, never calls LLMs), and Execute (tenex-agent, one-shot per turn, never op"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-07
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
---

# TENEX Role Separation

## Role Separation

TENEX enforces three strictly separated roles: Subscribe (relay), Orchestrate (runtime, never calls LLMs), and Execute (tenex-agent, one-shot per turn, never opens a relay connection). <!-- [^8a3eb-34] -->
