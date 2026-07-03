---
type: episode-card
date: 2026-07-03
session: a685f611-39bd-4a18-a6b7-ea4e38334b82
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/a685f611-39bd-4a18-a6b7-ea4e38334b82.jsonl
salience: root-cause
status: active
subjects:
  - relay-write-disabled
  - transport-error-mislabel
  - startup-publish-race
supersedes: []
related_claims: []
source_lines:
  - 73-116
  - 118-600
captured_at: 2026-07-03T11:02:59Z
---

# Episode: “Write actions are disabled” is a client-side relay-pool flag race, not a relay rejection

## Prior State

The `domain-event publish failed: relay rejected event: write actions are disabled` error was assumed to originate from the relay (`nip29.f7z.io`) being in a read-only or maintenance state, or from NIP-42 auth not completing before publish. A prior episode had attributed similar errors to daemon/CLI version skew (stale daemon vs. rebuilt CLI).

## Trigger

User asked to SSH into the relay host (pablo@157.180.102.242) and inspect its logs. Relay logs (journalctl for `nip29-f7z-io.service` / croissant) contained zero rejection or write-disabled messages. `strings` on the relay binary found no such phrase. Source tracing located the string in `nostr-relay-pool-0.44.1/src/relay/error.rs:168` as the `Display` text for `Error::WriteDisabled`.

## Decision

The error is entirely client-side: the daemon's in-memory `Relay` object for `nip29.f7z.io` has its `RelayServiceFlags` temporarily missing `WRITE`, so the SDK refuses to put the `EVENT` on the wire — the relay never sees it. The failures are intermittent within the same startup burst (some sessions publish fine, others fail), indicating a race in the relay-pool's internal flag state when ~10 reconciled sessions fire domain-event publishes simultaneously through the single shared `Transport`. The error log in `transport.rs:65` wraps this as “relay rejected event,” which is misleading — nothing was rejected.

## Consequences

- Relay-side investigation for this class of error is a dead end; the relay logs will never show these messages because they are never sent.
- The error is benign — session spawn proceeds regardless (spawning session engine follows each error).
- The `transport.rs` error message labeling should be corrected or enriched with relay-URL + flags context so future occurrences are diagnosable in place.
- The startup reconcile path may need publish serialization or retry to avoid the flag-race when many sessions revive simultaneously.
- A GitHub issue against rust-nostr may be warranted if the flag-race is an SDK bug rather than a usage issue.

## Open Tail

- Exact mechanism by which the WRITE flag is briefly unset during the startup burst is not yet pinpointed — no tenex-edge code explicitly strips WRITE.
- No code change has been made yet; the diagnosis is non-formal pending a decision on whether to serialize/retry, fix the log message, or file an upstream issue.

## Evidence

- transcript lines 73-116
- transcript lines 118-600

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-07-03-1-write-actions-are-disabled-is-a.json`](transcripts/2026-07-03-1-write-actions-are-disabled-is-a.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-07-03-1-write-actions-are-disabled-is-a.json`](transcripts/raw/2026-07-03-1-write-actions-are-disabled-is-a.json)
