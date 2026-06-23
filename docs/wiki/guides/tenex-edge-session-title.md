---
title: Tenex-Edge Session Title
slug: tenex-edge-session-title
topic: tenex-edge
summary: "The current session title is held in `cur_title: Option<String>`, a local variable in the per-session engine loop in `src/runtime.rs:149`, which is the single i"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:1b868736-ed6b-4f88-84d9-26bb320accfd
---

# Tenex-Edge Session Title

## In-Memory Authority

The current session title is held in `cur_title: Option<String>`, a local variable in the per-session engine loop in `src/runtime.rs:149`, which is the single in-memory authority while the session runs.

The in-memory `cur_title` is set in four places in `runtime.rs`: rehydrated from SQLite on startup (line 149), quick-seeded from the raw user prompt via `titleize_prompt` (line 266), overwritten by the distiller result (line 208), and never cleared otherwise (persists across idle).

The first message in a session shows an empty title because `turn_start` receives only the transcript path, and the runtime seed reads `read_last_user_prompt(transcript)` before Claude Code has flushed the submitted prompt to the transcript file, causing a one-turn lag. (Previously: The first message in a session must seed a title immediately from the prompt text passed through the turn_start RPC, not from the lagging transcript file read.)

The fix for the lagging title seed is to thread the prompt text through the `user-prompt-submit` hook into `turn_start`, persist it in a new `last_user_prompt` column + setter/getter in `state.rs`, and have the runtime seed prefer the captured prompt over the lagging transcript.

When the harness rotates the session id (resume/clear/compaction), the same conversation gets a new `session_id` with a new `cur_title` quick-seeded from the latest prompt, resulting in a second titled event while the old one never deletes or expires.

<!-- citations: [^1b868-3] [^1b868-4] [^1b868-5] [^1b868-21] [^1b868-45] -->
## Persistence

The in-memory `cur_title` is persisted to the `text` column of the `session_status` SQLite table (keyed by `pubkey`, `project`, `session_id`) via `s.set_agent_status`, and is rehydrated from this table on startup via `get_agent_status`. <!-- [^1b868-6] -->

## Wire Protocol

The title is published on the wire as the `['title', title]` tag of the kind:30315 event, keyed replaceable by `d = '<project>:<session_id>'`. <!-- [^1b868-7] -->
