---
type: episode-card
date: 2026-06-09
session: 435ec383-d607-459b-a712-a00ed4decaa7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/435ec383-d607-459b-a712-a00ed4decaa7.jsonl
salience: product
status: superseded
subjects:
  - session-id
  - short-id
  - send-message
supersedes: []
related_claims: []
source_lines:
  - 697-990
captured_at: 2026-06-17T23:58:37Z
---

# Episode: Session short IDs changed from UUID prefixes to hash-based codes to avoid confusion

## Prior State

Session IDs were displayed using the first 8 characters of the UUID (e.g., `019eac61`, `019eab61`), which were visually indistinguishable for UUIDs generated in the same time range (codex-generated UUIDs have very similar prefixes).

## Trigger

User observed that nearly identical truncated UUIDs made it impossible to distinguish sessions, and asked whether the IDs should be hashed instead.

## Decision

Replaced `short_id()` from simple UUID-truncation to a hash-based short code derived from the full session ID. Updated `send-message` recipient resolution to accept these hash codes as a lookup path (UUID prefix → hash code → agent slug).

## Consequences

- Displayed session codes are now visually distinct (e.g., `3e6862`, `83e647`, `5a6a31`) even for temporally adjacent UUIDs
- `send-message --recipient <hash>` works as a valid addressing mechanism alongside UUID prefixes and agent slugs
- All session-facing surfaces (`who`, `send-message`) now speak the same identifier format

## Open Tail

*(none)*

## Evidence

- transcript lines 697-990

