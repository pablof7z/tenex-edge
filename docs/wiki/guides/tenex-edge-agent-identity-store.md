---
title: tenex-edge Agent Identity Store
slug: tenex-edge-agent-identity-store
topic: tenex-edge
summary: The spawnable agents list is sourced from the tenex-edge agent identity store (~/.tenex/edge/agents/ JSON files), not from PATH `which` checks for binaries or f
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-17
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:bb7ee4ef-16bf-41b9-8e75-ed6b23f0f3a4
  - session:d683a556-03b8-4827-b84d-5395cd3610af
  - session:656e1e6b-2569-42da-8844-768a5e74788e
  - session:622711fa-5176-4580-b311-d66446c2924b
  - session:7cac50b6-a19d-4bd8-9be7-5c52aa8b2cca
  - session:d8d132f9-8a71-4af0-846c-44a4a9e01dc5
  - session:rollout-2026-06-16T14-11-38-019ed021-38a8-7472-bc5d-dc019a072086
  - session:rollout-2026-06-17T10-26-55-019ed479-d620-7de1-9f67-fcb327d70f95
  - session:ses_1307cfa82ffezNqP0fk6nYNJvs
---

# tenex-edge Agent Identity Store

## Agent Identity Store

The spawnable agents list is sourced from the tenex-edge agent identity store (~/.tenex/edge/agents/ JSON files), not from PATH `which` checks for binaries or from the selected project's NIP-29 member list. The `who` RPC call chain that populates the spawnable array is: TUI fetch_tui_data() -> RPC 'who' -> WhoSnapshot.spawnable -> tmux::spawnable_agents() -> identity::list_local_agents(edge_home). Local agent identities are read from ~/.tenex/edge/agents/*.json files to determine spawnable launch options, and an agent lacking NIP-29 project membership still appears as a launchable option because the menu displays local availability, not NIP-29 admission status.

Agent definition files (e.g., agents/slug.json) contain an `agent` field holding an inline JSON object with the agent definition, rather than referencing an external file path. The `command` field must be a JSON array (e.g., `["claude", "--dangerously-skip-permissions"]`), not a plain string, because `serde_json` silently deserializes a plain string for `Option<Vec<String>>` as `None`. (Previously: Agent identity files stored the harness command as an optional field `command: Option<Vec<String>>`, so each agent carries its own spawn recipe.)

A per-harness translator expands the agent definition into the correct CLI arguments for each harness binary. For the `claude` harness, the per-harness translator wraps the agent definition as `{ "<slug>": <def> }` and appends `--agents '<json>' --agent <slug>` to the spawn command. The translator function `apply_agent_def_args` is a no-op for harnesses other than `claude`.

The agent slug (derived from the JSON filename stem) is the identity displayed in the spawnable list, distinct from the harness command it spawns (e.g. slug 'developer' spawns 'claude --dangerously-skip-permissions').

The daemon's rpc_who uses `load_who_snapshot` in src/cli.rs; src/cli/who.rs is a dead code path and must not be the target for spawnable logic.

When spawning an agent, TENEX_EDGE_AGENT is passed via tmux -e (which propagates into the pane's environment) rather than via .env() on the tmux client (which is silently dropped and never reaches the pane).

The session-start hook (cli/hooks.rs:112) resolves the agent slug as TENEX_EDGE_AGENT or else the harness's self-reported default, so carrying TENEX_EDGE_AGENT via -e makes the spawn's known identity authoritative regardless of what the harness self-reports.

The root cause of a spawned codex agent showing up as claude is that spawn_agent passed TENEX_EDGE_AGENT via .env() on the tmux client process, which tmux silently drops, so the spawned pane never received the authoritative slug and the hook fell back to the harness's self-reported default (claude).

The agent field translation is applied at spawn time only; `resume_agent` uses the base command without re-appending agent definition arguments, since the session already has the agent definition.

Internally, the `StoredKey` struct includes an `agent: Option<serde_json::Value>` field for the inline agent definition. `list_local_agents` returns a 3-tuple of `(slug, command, agent_def)`. `resolve_spawn_entry` replaces `resolve_agent_command` and returns a `(base_cmd, agent_def)` tuple.

The label 'Spawnable (no session)' is renamed to 'Agents'.

The label 'spawnable via claude' is renamed to 'claude'.

Pressing Enter on a spawnable item in tenex-edge tmux spawns the agent, replacing the previous [n] key binding.

When a spawned harness reports `session_start`, the daemon makes a best-effort attempt to add the agent to the NIP-29 group by calling `provider.open_project(project, agent_pubkey)`.

The CLI provides an `agent` subcommand with `list`, `add`, `remove`, and `assign` actions for managing local agent keypairs.

`agent list` displays the slug, short pubkey, and spawn command for every local agent.

`agent add <slug> [-- <command>]` mints and persists a keypair if it does not exist, or updates the spawn command if it does, and is idempotent on re-runs.

`agent add <slug>` accepts a repeatable `--project <p>` flag to assign the newly-minted agent to one or more project NIP-29 groups in a single step.

`agent assign <slug> --project <p>` adds an already-existing local agent's pubkey to one or more project NIP-29 groups via the daemon's `project_add` RPC, with `--project` being required and repeatable.

Agent creation publishes the agent's kind:0 (profile) event, signed by the agent's own keys, to the indexer relay via a daemon RPC.

The kind:0 profile publish is best-effort upon `agent add`; if the daemon or relay is unavailable, the CLI prints a fallback message and the create still succeeds, deferring the publish to the agent's first session start.

Running `agent add` on an already-minted agent (to update its spawn command) does not re-publish the kind:0 profile.

`agent remove <slug>` soft-deletes the keypair by parking it at `<slug>.json.removed` rather than permanently unlinked, so the pubkey can be recovered with a simple rename.

Per-project assignment failures (e.g., operator key is not a group admin) are reported individually and do not abort the remaining project assignments.

`agent remove` on a missing slug prints to stderr but exits 0 for idempotent `rm -f`-style behavior.

The `tenex-edge` CLI treats the `opencode@laptop` target format as `agent@project` (meaning project 'laptop'), not `agent@host`.

Local agent keystore files are stored at `~/.tenex/edge/agents/<slug>.json` and contain a `public_key` field holding the agent's hex pubkey. <!-- [^ses_1-31] -->

The `list_local_agents` function reads all local agent keystore files and returns slug and command pairs, but does not return the public key. <!-- [^ses_1-32] -->

The `list_local_pubkeys` function reads public keys from the local keystore but does not associate them with agent slugs. <!-- [^ses_1-33] -->

The `who` command and spawn logic successfully find local agents by reading the local keystore directly via `identity::list_local_agents()`, bypassing `resolve_agent_pubkey`. <!-- [^ses_1-34] -->

The spawn-on-send logic resolves agents via the `sessions` table using `get_local_agent_slug_by_pubkey`, which only works for agents that have already run and registered. <!-- [^ses_1-35] -->

<!-- citations: [^rollo-75] [^62271-2] [^7cac5-1] [^7cac5-2] [^7cac5-3] [^7cac5-4] [^7cac5-5] [^7cac5-6] [^7cac5-7] [^7cac5-8] [^7cac5-9] [^7cac5-10] [^7cac5-11] [^bb7ee-1] [^bb7ee-2] [^bb7ee-3] [^bb7ee-4] [^d683a-1] [^d683a-2] [^d683a-3] [^656e1-1] [^d8d13-1] [^rollo-93] -->
