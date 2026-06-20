---
type: episode-card
date: 2026-06-16
session: 7cac50b6-a19d-4bd8-9be7-5c52aa8b2cca
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/7cac50b6-a19d-4bd8-9be7-5c52aa8b2cca.jsonl
salience: product
status: active
subjects:
  - agent-cli
  - local-keystore
supersedes: []
related_claims: []
source_lines:
  - 1-468
captured_at: 2026-06-18T00:46:39Z
---

# Episode: Local agent keystore management CLI

## Prior State

Agents were born lazily via load_or_create() on first harness use; no explicit add/remove/list commands existed. Removing an agent meant manually deleting its JSON file. Per-project agent scoping did not exist.

## Trigger

User asked whether a tool exists to add/remove locally-available agents from a project, and clarified the scope: 'what we need is a way to add/remove local agents (i.e. agents with a private key in .tenex/agents/<slug>.json)'. Project-agent mapping was clarified as already handled by the NIP-29 codec.

## Decision

Added `tenex-edge agent` CLI subcommand with: `list` (slug, pubkey, spawn command), `add <slug> [-- <command>]` (mint keypair, optionally set harness command; idempotent), `remove <slug>` (soft-delete to `.json.removed` to prevent irreversible key loss), `assign <slug> --project <p>` (add pubkey to NIP-29 group, repeatable for multiple projects), and `--project` flag on `add` for one-step mint + assign. Removal is soft so a re-minted key would be a different identity — parking preserves the original pubkey for recovery.

## Consequences

- Agents are explicitly manageable from the CLI rather than only through implicit first-use side effects
- Remove is a park-to-.removed rather than unlink, preventing accidental permanent loss of an agent's network identity (pubkey)
- Assign reuses the existing project_add daemon RPC, so admin/key requirements are identical to manual project add
- Per-project assignment failures don't abort the rest when multiple --project flags are given
- agent add --project ordering constraint: --project options must precede the -- separator

## Open Tail

- agent remove exits 0 on missing slug (rm -f idempotency) — user may want non-zero exit instead
- agent list shows truncated pubkey; full hex requires reading the JSON file — no --long flag yet

## Evidence

- transcript lines 1-468

