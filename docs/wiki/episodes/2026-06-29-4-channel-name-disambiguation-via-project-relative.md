---
type: episode-card
date: 2026-06-29
session: 661ebf6b-e01b-4ff6-b9c7-5042b900c788
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/661ebf6b-e01b-4ff6-b9c7-5042b900c788.jsonl
salience: product
status: active
subjects:
  - channel-resolution
  - command-routing
  - name-disambiguation
supersedes: []
related_claims: []
source_lines:
  - 142-147
  - 3456-3457
captured_at: 2026-06-29T10:05:11Z
---

# Episode: Channel name disambiguation via project-relative path resolution

## Prior State

Channel names globally unique within project. No hierarchical reference syntax. Ambiguous names resulted in undefined behavior or silent mis-routing (e.g., `tenex-edge channels switch planning` with multiple `#planning` channels).

## Trigger

User proposed: when ambiguity exists, 'show them "Which #planning within #current-project?" with alternatives'. Established that ambiguity requires user disambiguation, not silent resolution.

## Decision

Implemented `resolve_channel_ref` with project-relative path resolution: names resolve via suffix-matching to nearest `*/name` in scope; explicit `parent/child` syntax supported; opaque-id escape hatch `@<channel_h>` for force-by-id; ambiguity returns structured error (exit code 2) listing all matches with display names and copy-paste-ready re-run examples.

## Consequences

- Channels never leak opaque `channel_h` in user-facing output — all rendering by name only
- Ambiguous commands fail predictably with actionable suggestions instead of silent mis-routing
- Exit code 2 (non-standard) allows scripts to distinguish 'ambiguous' from 'failed'
- 4 unit tests for resolver edge cases

## Open Tail

- Collision fallback to `@id` mentioned in design but not yet implemented
- No interactive fuzzy picker for disambiguation — error message requires copy-paste

## Evidence

- transcript lines 142-147
- transcript lines 3456-3457

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-29-4-channel-name-disambiguation-via-project-relative.json`](transcripts/2026-06-29-4-channel-name-disambiguation-via-project-relative.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-29-4-channel-name-disambiguation-via-project-relative.json`](transcripts/raw/2026-06-29-4-channel-name-disambiguation-via-project-relative.json)
