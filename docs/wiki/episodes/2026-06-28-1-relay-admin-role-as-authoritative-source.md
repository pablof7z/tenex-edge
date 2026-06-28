---
type: episode-card
date: 2026-06-28
session: b9176726-a9a8-41a9-b806-c966e8c94ed7
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/b9176726-a9a8-41a9-b806-c966e8c94ed7.jsonl
salience: architecture
status: active
subjects:
  - group-management-authority
  - owns-group-flag
  - relay-admin-consistency
supersedes:
  - 2026-06-09-1-daemon-actively-owns-nip-29-group
related_claims: []
source_lines:
  - 275-278
  - 281-293
  - 407-428
captured_at: 2026-06-28T07:23:25Z
---

# Episode: Relay admin role as authoritative source for group management

## Prior State

Group management decisions (member warnings, per-session room rename) are gated on owned_groups.owns_group, a local boolean set when this daemon instance creates a group. This conflates group creation with administrative authority.

## Trigger

User identifies that NIP-29 kind:39001 (relay admin list, materialized in group_members.role='admin') should be the authoritative source for management capability, yet code paths diverge: readiness.rs correctly consults the relay while turn.rs:88, runtime.rs:251, and chat_publish.rs:103 ignore it in favor of the local flag. Empirical case: #29er was created by this backend but is absent from owned_groups, breaking management checks despite relay confirming admin status.

## Decision

Repoint all group management gates to consult group_members where role='admin' (materialized from relay 39001 snapshots). Add fallback live relay fetch when group_members cache is empty. Remove owns_group from all three failing code paths; demote it to optional creation-time hint only.

## Consequences

- Three code paths now correctly use relay authority, fixing false-negative member warnings (turn.rs:88) and silent per-session room rename failures (runtime.rs:251, chat_publish.rs:103)
- Cache misses like #29er now trigger live relay fetch instead of silent management failure
- Multi-daemon scenarios become consistent: relay authority is never overridden by divergent local creation records
- owns_group column is demoted from authoritative gate to optional observational hint

## Open Tail

- Decision: fully delete owns_group column or retain for creation-time observability?
- Implementation: live 39001 fetch fallback in ensure_channel_ready when group_members is empty

## Evidence

- transcript lines 275-278
- transcript lines 281-293
- transcript lines 407-428

## Conversation

- Cleaned transcript (verbatim user words, abbreviated agent replies): [`transcripts/2026-06-28-1-relay-admin-role-as-authoritative-source.json`](transcripts/2026-06-28-1-relay-admin-role-as-authoritative-source.json)
- Raw transcript (verbatim user words, full agent replies): [`transcripts/raw/2026-06-28-1-relay-admin-role-as-authoritative-source.json`](transcripts/raw/2026-06-28-1-relay-admin-role-as-authoritative-source.json)
