---
title: Tenex-Edge Activity Distillation
slug: tenex-edge-activity-distillation
topic: tenex-edge
summary: Activity distillation is LLM-based, with an optional heuristic gate to bound cost
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-09
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
---

# Tenex-Edge Activity Distillation

## Activity Distillation

Activity distillation is driven by the conversation transcript, not by tool-use events. It uses an LLM-only approach with no heuristic fallback; HeuristicDistiller and all no-LLM fallbacks are removed. The `ToolEvent` model and `obs` table are removed from the codebase. The `distill_activity` function takes only the transcript as input, removing the tool events parameter. It is automatic (auto-distill, not manual) because agents will forget to call it if left to them. A turn-start hook (e.g., UserPromptSubmit, carrying session + transcript_path) initiates the activity tracking lifecycle instead of session-start or tool-use events. A lock prevents re-arming the poll timer while the agent is already working within the same turn; one timer is armed per turn. The first distillation check fires at 30 seconds, then loops every 5 minutes for long-running turns. A turn that finishes within 30 seconds produces zero LLM distillation calls. Published Status is refreshed on the 30-second heartbeat (without re-running the LLM) to prevent it from expiring during long turns between 5-minute distillation intervals. A turn-end hook (e.g., Stop) clears the polling timer and lock, setting the agent status back to idle. The engine's existing select! loop owns the timer; hooks flip state rather than spawning separate OS polls. The store holds a per-session working-state record containing {working: bool, turn_started_at, transcript_path, distilled_this_turn: bool} replacing obs rows. The distiller reads the conversation transcript to produce intent-level summaries; Claude Code and Codex use the file transcript via `transcript_path`, OpenCode fetches from the SDK message store and writes a temp JSONL. The distiller uses a direct API call to a cheap model (e.g. openai/gpt-4o-mini via OpenRouter, not the claude CLI, which would re-trigger hooks recursively). The distiller is case-insensitive for tool names (e.g., Claude 'Edit' vs OpenCode 'edit'). Distillation configuration lives in `~/.tenex/providers.json` and `~/.tenex/llms.json` using the existing TENEX format, with an `edge-distillation` role selecting a named model; the LLM call is done natively via rig.rs supporting both openrouter and ollama providers. Agent activity is published as kind:1 with a NIP-29 `h` tag for the project slug, and the content describes what the agent is doing. Agents keep a running NIP-38 status (kind: 30315), also h-tagged to the project slug, empty when idle.

<!-- citations: [^f3a73-94] [^f3a73-47] [^f3a73-66] [^f3a73-75] [^f3a73-80] [^f3a73-86] [^f3a73-93] [^f3a73-100] [^f3a73-104] [^95659-1] [^95659-4] -->
