---
type: episode-card
date: 2026-06-17
session: 52474db7-1e81-4011-a859-6343bfeae807
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/52474db7-1e81-4011-a859-6343bfeae807.jsonl
salience: product
status: active
subjects:
  - distillation-context
  - session-title
  - transcript-extract
supersedes:
  - 2026-06-14-2-session-distillation-engine-immediate-title-seeding
related_claims: []
source_lines:
  - 41-46
  - 129-138
  - 189-203
  - 425-443
captured_at: 2026-06-18T00:51:52Z
---

# Episode: Strip tool_use from distillation transcript to anchor titles on user intent

## Prior State

The extract() function in transcript.rs included tool_use blocks (e.g. [uses Read src/codec/kind1.rs]) in the assistant portion of the transcript fed to the distillation LLM. When the distiller fired early in a turn (before the agent had produced any text), the only assistant content visible was tool calls, causing the model to anchor on the agent's mechanical actions and generate titles like "Analyze Rust code for specific structures" rather than reflecting the user's stated intent.

## Trigger

User reported that a session titled "Analyze Rust code for specific structures" should have been "Store explicit user messages in episodes" — the model observed what the agent started doing and used that as the title instead of the user's intention. Comparative test with Claude directly produced the correct title from the same prompt.

## Decision

Removed the tool_use branch from extract() in transcript.rs so assistant messages in the distillation context only contain text blocks, not tool-use hints. When the distiller fires at 3 seconds and the agent has only issued tool calls (no text yet), the transcript now shows only the user message, making it the dominant signal. The initially-attempted last_user_prompt parameter addition to distill_session was reverted as unnecessary once tool_use stripping alone proved sufficient.

## Consequences

- Early-turn distillation now produces titles reflecting user intent rather than agent actions, validated across multiple experimental scenarios (generic grep calls, wrong files, vague user messages, nudge-to-keep)
- The existing test asserting [uses Edit src/auth.rs] appears in read_recent output was updated to assert its absence
- The distill_session signature change was started then reverted — the simpler approach (strip at extraction time) was sufficient

## Open Tail

*(none)*

## Evidence

- transcript lines 41-46
- transcript lines 129-138
- transcript lines 189-203
- transcript lines 425-443

