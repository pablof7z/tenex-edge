---
type: episode-card
date: 2026-06-12
session: 081ec521-c99b-42fb-9aa7-4a109519a62f
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/081ec521-c99b-42fb-9aa7-4a109519a62f.jsonl
salience: product
status: active
subjects:
  - tenex-edge-hooks
  - nip29-groups
  - turn-context
  - agent-warning
supersedes: []
related_claims: []
source_lines:
  - 1385-1403
  - 1560-1604
captured_at: 2026-06-18T00:08:14Z
---

# Episode: Imperative NIP-29 membership warning on first agent turn

## Prior State

No warning existed when an agent's pubkey wasn't in the project's NIP-29 group. Agents could silently have their messages dropped by the relay with no indication.

## Trigger

The remote-machine Claude Code session started, received a membership warning in hook output, but treated it as background context and did not surface it to the user. User observed: 'the wording is not strong enough' — the agent just said 'hi' and ignored the warning.

## Decision

Added a membership check in `assemble_turn_start_context` that emits a warning block when `is_group_member` returns false for the session's agent. The initial wording ('WARNING: this agent is not a member…') was then revised after live testing showed agents ignored it. Final wording uses imperative language: 'ACTION REQUIRED — your FIRST response to the user MUST include this warning verbatim' and 'Do not proceed with any other task until the user acknowledges this.'

## Consequences

- Agents on fresh machines (with empty local membership caches) immediately get told to have the operator run `tenex-edge project add`
- The warning includes the exact command and the agent's hex pubkey, making it copy-paste actionable
- Warning is scoped to the first turn only, avoiding repeated noise once the daemon cache populates
- Live test on remote machine confirmed the warning fires; the relay returned 'all targets are members already' on `project add`, revealing that the stale-cache case is the primary trigger

## Open Tail

- The warning can fire spuriously when the local cache is empty but the relay already has the member — the check should ideally also query kind:39002 from the relay rather than relying solely on local state

## Evidence

- transcript lines 1385-1403
- transcript lines 1560-1604

