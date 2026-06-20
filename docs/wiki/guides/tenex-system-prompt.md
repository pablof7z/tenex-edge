---
title: TENEX System Prompt
slug: tenex-system-prompt
topic: agent-configuration
summary: TENEX's system prompt is deterministic and serves as a cache anchor; all per-turn volatile material (reminders, RAG hits, todos) goes into the projected user me
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

# TENEX System Prompt

## System Prompt Architecture

TENEX's system prompt is deterministic and serves as a cache anchor; all per-turn volatile material (reminders, RAG hits, todos) goes into the projected user message, never the system prompt. <!-- [^8a3eb-36] -->
