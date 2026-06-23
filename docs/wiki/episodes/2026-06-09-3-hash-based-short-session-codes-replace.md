---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: product
status: active
subjects:
  - session-id-display
  - recipient-resolution
supersedes: []
related_claims: []
source_lines:
  - 697-700
  - 734-768
  - 919-989
captured_at: 2026-06-12T20:14:05Z
---

# Episode: Hash-based short session codes replace UUID-prefix truncation

## Prior State

Session IDs were displayed as the first 8 characters of their UUID, which produced near-identical strings for codex-generated sessions (e.g., `019eac61`, `019eab61`, `019eab5b`), making them indistinguishable to users.

## Trigger

User observed that three different codex sessions had visually indistinguishable 8-char prefixes and asked for a better identification scheme.

## Decision

Replaced `short_id()` (UUID prefix truncation) with a hash-based 6-character hex code derived from the full session ID. Updated `resolve_recipient()` to support hash-code lookup as a fallback, ensuring `send-message` can route using the same codes displayed by `who`.

## Consequences

- Session codes are now visually distinct (e.g., `3e6862`, `83e647`, `5a6a31` instead of `019eac61`, `019eab61`, `019eab5b`).
- `send-message --recipient <hash-code>` works as a resolution path alongside UUID prefix and agent slug.
- Resolution order: UUID prefix → hash code → agent slug.

## Open Tail

*(none)*

## Evidence

- transcript lines 697-700
- transcript lines 734-768
- transcript lines 919-989

