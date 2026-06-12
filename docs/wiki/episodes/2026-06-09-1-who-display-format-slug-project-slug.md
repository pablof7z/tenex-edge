---
type: episode-card
date: 2026-06-09
session: 240ffb86-8827-4741-932b-29fb1824c0c7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/240ffb86-8827-4741-932b-29fb1824c0c7.jsonl
salience: product
status: active
subjects:
  - who-command
  - agent-identifier
  - slugify-host
supersedes: []
related_claims: []
source_lines:
  - 64-79
  - 1167-1209
captured_at: 2026-06-12T19:59:53Z
---

# Episode: who display format: slug@project → slug@hostname

## Prior State

Agent identities displayed as `slug@project` (e.g., `claude@tenex-edge`). Raw host names like "pablos' laptop" used as-is in the @slot.

## Trigger

User correction: "it should be agentSlug@hostname" (hostname from `backendName` in config), then "the hostname slug should be sluggified otherwise an agent trying to send a message to another agent might be confused on what to use (codex@pablo's laptop? codex@pablo?)"

## Decision

Display format changed to `slug@hostname` where hostname is `slugify_host(backendName)` — lowercases, replaces non-alphanumeric with `-`, collapses consecutive hyphens, falls back to "unknown". Project shown as a dimmed secondary field. Raw `backendName` still stored and published to Nostr unchanged.

## Consequences

- Agent-to-agent addressing is unambiguous (canonical slug form)
- Project info still visible but demoted to secondary field
- HOST column removed from live view; merged into AGENT column as `slug@hostname`
- Tests updated: assert `@laptop` instead of `@proj`
- `slugify_host` added to `util.rs` with test cases

## Open Tail

*(none)*

## Evidence

- transcript lines 64-79
- transcript lines 1167-1209

