---
title: Tenex-Edge Wait-for-Mention
slug: tenex-edge-wait-for-mention
topic: tenex-edge
summary: The `wait-for-mention` command polls the SQLite inbox every ~500ms until a mention arrives
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:162f9965-82ca-420b-aa24-99faa15cb59a
---

# Tenex-Edge Wait-for-Mention

## Polling and Inbox Behavior

The `wait-for-mention` command is replaced by a channels-based approach for injecting async work. Tenex-Edge uses channels (notifications/claude/channel) instead of the wait-for-mention hack. (Previously: The `wait-for-mention` command polled the SQLite inbox every ~500ms until a mention arrived.)

<!-- citations: [^3da7f-6] [^3da7f-7] [^3da7f-8] [^162f9-21] [^162f9-28] -->
