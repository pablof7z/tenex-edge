---
type: episode-card
date: 2026-06-10
session: 56f9fe89-5ff7-4e5b-b202-334cd7629d42
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/56f9fe89-5ff7-4e5b-b202-334cd7629d42.jsonl
salience: reversal
status: active
subjects:
  - propose-command
  - kind-30023
  - agent-tags
supersedes: []
related_claims: []
source_lines:
  - 807-1457
captured_at: 2026-06-18T00:06:08Z
---

# Episode: Restore accidentally-deleted propose command without agent tags or session requirement

## Prior State

The `propose` CLI verb and `rpc_propose` daemon handler (publishing kind:30023 long-form articles) existed in the codebase but were accidentally dropped during the `98582fa` file-size-limit refactor that split cli.rs and server.rs into submodules. The old version required a live session and included `agent` tags.

## Trigger

User noticed the command was gone ('what happened with the command that published proposals as 30023?') and then the restored version included agent tags, which the user explicitly rejected.

## Decision

Restored `propose` as a CLI verb + `rpc_propose` daemon RPC handler publishing kind:30023 events. Key changes from the lost version: (1) no `agent` tags — removed entirely per the 'agent tags must never exist' invariant; (2) works without a live session — falls back to cwd for project and `TENEX_EDGE_AGENT` for slug, omitting the `session-id` tag when no session is active; (3) no thread dual-write to local DB (that infrastructure was removed in prior commits).

## Consequences

- Propose is usable by agents without an active session, matching the pattern of other CLI verbs.
- The `--thread` flag adds an NIP-10 `e` root tag linking the proposal to a conversation.
- No agent tags are stamped on any published event — the codec still has two agent-tag lines in kind1.rs that remain to be cleaned up.
- The thread dual-write (local SQLite read-model) is not restored; proposals are published to relay only.

## Open Tail

- Agent tags still exist at codec/kind1.rs:174 and codec/kind1.rs:200 — should be removed per the invariant.

## Evidence

- transcript lines 807-1457

