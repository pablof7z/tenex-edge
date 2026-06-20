---
type: episode-card
date: 2026-06-12
session: cd74a605-9f83-4e21-a885-4d900e88ce07
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/cd74a605-9f83-4e21-a885-4d900e88ce07.jsonl
salience: product
status: active
subjects:
  - inbox-envelope-format
  - inbox-command-surface
  - mention-metadata
supersedes: []
related_claims: []
source_lines:
  - 69-218
  - 700-780
  - 912-927
  - 1289-1382
captured_at: 2026-06-18T00:10:40Z
---

# Episode: Inbox messages redesigned as email-like envelopes with unified command surface

## Prior State

Inbox messages displayed as one-line summaries (only from_slug, project, reply_to, body). Sending used a standalone `send-message` command. No subject line, no sender git context, no timestamp display, no reply-by-ID, no remote-host annotation.

## Trigger

User directive: 'it needs to be much more of a message with a proper envelope, like an email' — with explicit format spec (From with session + optional remote host, Date with relative time, Branch/commit/dirty-count, ID for reply, Subject). Follow-up refinements: fixed separator (two dashes), 'just now' <1 min, capture sender's workspace git state at send time, include message ID for reply, add sender session ID to From line.

## Decision

Messages now render as email-like envelopes via a single `format_envelope` renderer used everywhere (CLI inbox, wait-for-mention, turn-injection). CLI unified under `inbox` (replacing `send-message`) with `inbox send --to --subject` and `inbox reply --id` (NIP-10 e-tag + p-tag reply). Sender's git workspace (branch, short commit hash, dirty-file count) and subject captured at send time, carried on the wire as new MentionMeta/kind:1 tags, and persisted in new inbox table columns. Remote senders annotated with `[remote: host]`.

## Consequences

- Schema migration adds columns (subject, git_branch, commit_hash, git_dirty, from_host) to inbox table
- Wire protocol gains new kind:1 tags: subject, git-branch, git-commit, git-dirty, from-host
- `send-message` CLI command removed with no backward-compat shim
- Daemon must be restarted to pick up new `inbox_reply` RPC and envelope JSON fields
- All three mention render paths (inbox, wait-for-mention, turn-injection) now produce identical envelope output
- Dirty-count uses `git status --porcelain` minus gitignored files; omitted entirely when zero, singular/plural 'file/files'
- libc added as dependency for local-time formatting (localtime_r)

## Open Tail

- Running daemon still serves old binary — needs restart before new commands work end-to-end
- MCP `reply` tool still calls `send_message` RPC (now with optional subject); may want a dedicated MCP inbox-reply tool later

## Evidence

- transcript lines 69-218
- transcript lines 700-780
- transcript lines 912-927
- transcript lines 1289-1382

