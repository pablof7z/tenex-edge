---
type: episode-card
date: 2026-06-12
session: 081ec521-c99b-42fb-9aa7-4a109519a62f
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/081ec521-c99b-42fb-9aa7-4a109519a62f.jsonl
salience: product
status: active
subjects:
  - tenex-edge-cli
  - nip29-groups
  - project-add
supersedes: []
related_claims: []
source_lines:
  - 446-560
  - 1062-1093
captured_at: 2026-06-18T00:08:14Z
---

# Episode: Add `project add` CLI command for NIP-29 group membership

## Prior State

No CLI command existed to manually add an agent to a NIP-29 project group. The only path was automatic addition during session-start (daemon calls `group_put_user` if the operator key is admin), and there was no way for an operator to add members from another machine or add by npub/nip05.

## Trigger

User asked whether new agents request group access and whether there was any way to add them from tenex-edge. Investigation confirmed no `add-member` subcommand existed — only `list` and description-setting under `project`.

## Decision

Added `tenex-edge project add <project> <pubkey-or-npub-or-nip05>` CLI subcommand that resolves hex/npub via `PublicKey::parse` and NIP-05 via HTTP fetch, then publishes a kind:9000 `put-user` event signed by the operator's `userNsec`. Wired through a new `ProjectAction::Add` variant in `cli.rs`, a `project_add` dispatch in `cli/admin.rs`, and a `rpc_project_add` RPC handler in `daemon/server/admin.rs` with route in `connection.rs`. Added `reqwest` dependency for NIP-05 resolution.

## Consequences

- Operators can now add arbitrary pubkeys (hex, npub, or NIP-05) to project groups from any machine with relay admin access
- The daemon resolves NIP-05 addresses over HTTPS before publishing, so human-friendly identifiers work
- The relay may reject the event if the signing key isn't a group admin, producing a clear error

## Open Tail

*(none)*

## Evidence

- transcript lines 446-560
- transcript lines 1062-1093

