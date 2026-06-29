---
type: episode-card
date: 2026-06-29
session: c55adda0-b071-4b76-9d24-a0cbcb5b6e0c
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/c55adda0-b071-4b76-9d24-a0cbcb5b6e0c.jsonl
salience: product
status: active
subjects:
  - awareness-other-active-channels
  - unnamed-channel-filtering
supersedes: []
related_claims: []
source_lines:
  - 1-8
  - 130-162
captured_at: 2026-06-29T10:53:41Z
---

# Episode: Skip unnamed channels in awareness output

## Prior State

The 'Other active channels, last 10m:' section in awareness output included unnamed channels (session rooms whose name is empty or equals their own id), displaying them as '(unnamed channel) [N members]'. These occupied slots in the top-5 list without providing useful information.

## Trigger

User explicitly stated unnamed channels are useless and should be skipped — there is no value for an agent to know about them in this context.

## Decision

Unnamed channels are now filtered out in `other_active_channel_lines` before the `take(5)` cap is applied, so they never appear in the awareness block. An `is_named_channel` helper was added near `channel_label` to encapsulate the unnamed-detection logic (name empty or equals its own id).

## Consequences

- The top-5 'Other active channels' slots are now reserved for named channels only, potentially surfacing more useful channels that were previously pushed out by unnamed ones.
- A reusable `is_named_channel` predicate is now available for any future code that needs to distinguish named from unnamed channels.
- Existing tests asserting absence of '(unnamed channel)' (line 168 of tests.rs) are now satisfied at the source rather than only via the take(5) cap.

## Open Tail

*(none)*

## Evidence

- transcript lines 1-8
- transcript lines 130-162

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-skip-unnamed-channels-in-awareness-output.json`](transcripts/2026-06-29-1-skip-unnamed-channels-in-awareness-output.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-skip-unnamed-channels-in-awareness-output.json`](transcripts/raw/2026-06-29-1-skip-unnamed-channels-in-awareness-output.json)
