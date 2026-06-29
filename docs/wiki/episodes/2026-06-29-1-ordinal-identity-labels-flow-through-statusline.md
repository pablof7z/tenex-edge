---
type: episode-card
date: 2026-06-29
session: d0db1eb1-0eb3-4ab5-9a9f-93a1779283a2
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/d0db1eb1-0eb3-4ab5-9a9f-93a1779283a2.jsonl
salience: product
status: superseded
subjects:
  - ordinal-identity
  - statusline
  - kind-0-publish
  - engine-params
supersedes: []
related_claims: []
source_lines:
  - 1-1
  - 88-172
  - 173-173
  - 407-662
captured_at: 2026-06-29T10:00:30Z
---

# Episode: Ordinal identity labels flow through statusline and kind:0 publish

## Prior State

Statusline RPC returned raw agent_slug ('claude') for all sessions regardless of ordinal allocation. Kind:0 events published by ordinal sessions used session_label format instead of ordinal labels ('claude1', 'claude2'). Both paths ignored the identities table where ordinal allocations are stored.

## Trigger

User observed two sessions showing identical base agent names in statusline. User then required kind:0 publish to also use ordinal labels (e.g. 'claude1').

## Decision

Added agent_label field to EngineParams struct to carry ordinal labels. Modified statusline RPC to look up identity_for_session and render ordinal labels matching who command. Updated session_start and engine_lifecycle to pass signer.label to engine_params_for. Modified runtime to use agent_label when publishing kind:0 profile events.

## Consequences

- Statusline now correctly shows ordinal suffixes ('claude1' for ordinal 1, vs 'claude' for ordinal 0)
- Kind:0 events from ordinal sessions now publish correct name field visible to other agents on relay
- Identity representation is now consistent across statusline query, kind:0 publish, and all agent-visible signals
- Ordinal label is now first-class in EngineParams rather than computed ad-hoc at multiple sites

## Open Tail

*(none)*

## Evidence

- transcript lines 1-1
- transcript lines 88-172
- transcript lines 173-173
- transcript lines 407-662

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-1-ordinal-identity-labels-flow-through-statusline.json`](transcripts/2026-06-29-1-ordinal-identity-labels-flow-through-statusline.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-1-ordinal-identity-labels-flow-through-statusline.json`](transcripts/raw/2026-06-29-1-ordinal-identity-labels-flow-through-statusline.json)
