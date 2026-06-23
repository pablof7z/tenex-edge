---
title: Tenex-Edge Thread Storage
slug: tenex-edge-thread-storage
topic: tenex-edge
summary: The thread dual-write infrastructure (local SQLite read-model of relay conversations via `projects`, `threads`, `messages`, `message_recipients` tables) was rem
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-10
updated: 2026-06-10
verified: 2026-06-10
compiled-from: conversation
sources:
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:40a4d401-2520-4781-b747-b0ef19594bed
---

# Tenex-Edge Thread Storage

## Thread Storage History

The thread dual-write infrastructure (local SQLite read-model of relay conversations via `projects`, `threads`, `messages`, `message_recipients` tables) was removed from the codebase prior to the `98582fa` refactor. Each session tracks thread_root_event_id (immutable, set on first user prompt) and last_prompt_event_id (updated every user prompt) in the database.

<!-- citations: [^56f9f-9] [^40a4d-19] -->
