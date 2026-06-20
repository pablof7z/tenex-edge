---
title: tenex-edge Distillation
slug: tenex-edge-distillation
topic: tenex-edge
summary: Distillation is automatic (auto-distill), not manual; agents are not relied on to call it themselves
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-17
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:215d979a-a054-4e2b-b349-851e0d874d6d
  - session:633f8f7f-37f8-409c-90a9-ef64b0dc3216
  - session:52474db7-1e81-4011-a859-6343bfeae807
---

# tenex-edge Distillation

## Distillation Mode

Distillation is automatic (auto-distill), not manual; agents are not relied on to call it themselves. Activity distillation is driven by conversation transcripts, not tool use events. Activity distillation uses LLM-only analysis of the transcript with no heuristic fallback. (Previously: the engine used a heuristic fallback.)

Session distillation runs in a background tokio task with a 20-second timeout so slow API responses never block heartbeats or presence updates. <!-- [^215d9-1] -->

<!-- citations: [^f3a73-8] [^f3a73-9] [^95659-3] -->
## Transcript Handling

Distillation uses the conversation transcript (not just tool names), exactly like pc. The `extract()` function in `src/transcript.rs` excludes `tool_use` blocks from assistant messages, so the distillation transcript only contains text blocks from the assistant. The store tracks per-session turn state (`working`, `turn_started_at`, `transcript_path`, `distilled_this_turn`) instead of tool events. The `distill_activity` function takes only the transcript as input, with no tool event parameter. The engine loop uses a liveness tick to check turn state rather than a burst buffer. Status (NIP-38) is refreshed on the 30-second heartbeat (without LLM) to prevent TTL expiry during long turns between distillation cycles. The removed tool-driven path includes the `observe` verb, `ToolEvent` type, `HeuristicDistiller`, `obs` table, `distill_min` gate, and `llm_available` flag.

The session title distillation prompt in `src/distill.rs` is left unchanged; the prompt itself is correct and the root problem was tool-use noise in the context rather than prompt wording. The `last_user_prompt` parameter is not added to `distill_session`; the earlier signature change in `src/distill.rs` was reverted because stripping tool-use alone provides a sufficient user-intent anchor.

The `titleize_prompt` helper extracts the first non-empty line of the prompt, strips leading markdown/list prefixes, truncates to 60 characters at a word boundary.

The `read_last_user_prompt` function tails the last 96KB of the transcript JSONL, finds the last user message, skips pure tool_result messages, and returns the raw text.

The first title published to the 30315 event is generated immediately on turn start via `titleize_prompt` on the raw user message (titlecase + 60-char word-boundary truncation), followed by an LLM-quality title ~30 seconds later via `distill_session`.

<!-- citations: [^215d9-3] [^215d9-4] [^633f8-1] [^f3a73-10] [^f3a73-11] [^95659-4] [^52474-3] -->
## Throttling

The first distillation check occurs 30 seconds after a turn starts, and subsequent checks loop every 5 minutes while the turn continues. A turn that finishes within the initial 30-second window produces zero LLM distillation calls. A lock prevents re-arming the timer or stacking polls within the same working turn — only one armed timer per turn. A sub-threshold turn (finishing before the first-check interval) publishes zero distillation — verified as the headline behavior.

A failed distillation attempt does not consume the retry slot: `last_distill` is only updated on success, and a separate `last_distill_attempt` timestamp enables retry after another `turn_first` window. <!-- [^215d9-2] -->

<!-- citations: [^f3a73-13] [^95659-5] -->

## Model Configuration

The `edge-distillation` role (which generates the 30315 title) is configured to use `openrouter/openai/gpt-4o-mini` via OpenRouter, while the default model for everything else is `kimi-k2.6:cloud` via Ollama. <!-- [^633f8-2] -->

## Error Handling

Distillation LLM errors are surfaced both by appending timestamped lines to `~/.tenex/edge/logs/<session_codename>.log` (e.g. `bravo4217.log`) and by recording them in a `session_errors` database table so the statusline can display them. When the distill task finishes with an error, runtime.rs appends a timestamped line to `~/.tenex/edge/logs/<codename>.log` and records the error in the database for the statusline. `distill_session` returns `(Option<SessionLabels>, Option<String>)` where the error string is `Some` only when the LLM was actually called and failed, not for nudge-to-keep or empty transcripts. <!-- [^633f8-3] -->
