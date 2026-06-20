---
title: tenex-edge Session Forensics Logging
slug: tenex-edge-session-forensics-logging
topic: tenex-edge
summary: JSONL logs are written per-session under ~/.tenex/edge/sessions/<session-id>/ with separate hook-calls.jsonl and command-calls.jsonl files, instead of a single
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-17
updated: 2026-06-17
verified: 2026-06-17
compiled-from: conversation
sources:
  - session:3b87cdd2-dc84-40d5-9bf0-677e282fe0e4
  - session:rollout-2026-06-16T13-48-03-019ed00b-a229-7a31-b08c-ec839e243f28
  - session:rollout-2026-06-17T10-44-56-019ed48a-54f6-7c41-a23b-dfde9dc65c2f
---

# tenex-edge Session Forensics Logging

## Per-Session Log Layout

JSONL logs are written per-session under ~/.tenex/edge/sessions/<session-id>/ with separate hook-calls.jsonl and command-calls.jsonl files, instead of a single monolithic file for all sessions. Old monolithic hook-calls.jsonl and command-calls.jsonl files are deleted; new events go exclusively to the per-session layout. Every raw argv is recorded to ~/.tenex/edge/command-calls.jsonl before clap parsing, so even hallucinated or invalid subcommands leave a forensic record. Any failed invocation of tenex-edge by an agent (including hallucinated session IDs or hallucinated commands) must create a record.

<!-- citations: [^3b87c-1] [^rollo-65] [^rollo-104] -->
## Writer Implementation

hook_forensics.rs extracts session_id from stdin JSON (checking session_id, sessionId, conversation_id, etc.) at start() time and writes to sessions/<id>/hook-calls.jsonl. command_forensics.rs reads TENEX_EDGE_SESSION env (or --session flag) and writes to sessions/<id>/command-calls.jsonl. Every raw argv is recorded before clap parsing, so even hallucinated or invalid subcommands leave a forensic record. When no session_id is available, both writers fall back to sessions/_unscoped/. Both hook and command writers still honor the TENEX_EDGE_HOOK_CALL_LOG and TENEX_EDGE_COMMAND_CALL_LOG override environment variables.

<!-- citations: [^3b87c-2] [^rollo-66] [^rollo-67] [^rollo-68] [^rollo-105] -->
## Reader Behavior

The debug reader enumerates sessions/*/ directories and reads each session's two files in full with no byte-limit tail, since per-session files stay small. The directory name is used as session_hint fallback for entries that lack an explicit session_id (early hook events before the field is set by Claude Code). The reader falls back to old monolithic files if the sessions/ directory doesn't exist yet, preserving backward compatibility. <!-- [^3b87c-3] -->
