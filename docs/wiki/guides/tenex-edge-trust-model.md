---
title: tenex-edge Trust Model
slug: tenex-edge-trust-model
topic: tenex-edge
summary: The trust model authorizes events by signer pubkey plus NIP-29 relay-authoritative group membership; NIP-29 relay state is the single source of truth for projec
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-16
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:98f9939c-f42b-43dd-baba-d9a176d4b2d7
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:40a4d401-2520-4781-b747-b0ef19594bed
  - session:081ec521-c99b-42fb-9aa7-4a109519a62f
  - session:rollout-2026-06-14T13-17-10-019ec5a2-a38e-7403-906f-836d766d9291
  - session:rollout-2026-06-16T12-40-33-019ecfcd-d47b-7992-998f-75432d8ac4cf
---

# tenex-edge Trust Model

## Trust Model

The trust model authorizes events by signer pubkey plus NIP-29 relay-authoritative group membership; NIP-29 relay state is the single source of truth for project metadata and group membership. The 'agent' wire tag is entirely rejected—it must not be written or read. Message admission and routing gates on signer ∈ hosted ∪ owners ∪ group members, with no local ACL allow/block system (whitelisted-agents.txt / blocked-agents.txt) involved in the NIP-29 codec—those files are exclusively used by the kind:1 codec. Authorization comes from signer pubkey plus hydrated NIP-29 membership/owner/hosted checks, not from session IDs. The `tenex-edge acl` CLI subcommand does not exist, nor does the `Acl` tail event category or its renderer/filters. The `pending_agents` database table and its store accessors are removed; new code does not create, write, read, or expose it. The slug always resolves from the signer's kind:0 profile for both owners and agents, with no special case; profile materialization persists delivered profiles directly without branching on local allow/block files, and daemon subscriptions for profiles do not include an owner-p-tag discovery filter. An owner-signed note published straight to nip29.f7z.io (no agent tag, signed by userNsec) was verified to land in codex's live session inbox on the production daemon, resolving the sender name from kind:0 profile ('Pablo Testing Pubkey'). publish_signed signs an event with a specific keypair and publishes it over the daemon's shared relay connection; a B-signed event published over an A-authenticated connection lands under B's authorship. (Previously: trust was scoped via explicit roster attestation (kind:34199) and scoped, revocable capability grants (kind:4120), with an allowlist of agent pubkeys at ~/.tenex/whitelisted-agents.txt.) New agents are added directly via the daemon using group_put_user (kind:9000) at session-start, relying on the daemon's key being a group admin, rather than requesting NIP-29 group access via a join-request (kind:9021) flow. The `tenex-edge project add <project> <pubkey-or-npub-or-nip05>` subcommand (dispatched via rpc_project_add RPC under ProjectAction) resolves the identifier (hex, npub/bech32, or NIP-05 via HTTP fetch), publishes a kind:9000 put-user event signed by userNsec, caches the membership, and returns the result. On the first UserPromptSubmit of a session, the turn-start context checks for membership: the 'not a member' warning fires only when a membership roster is known and the agent is absent, not when membership has not yet hydrated. When the warning fires, it injects a blocking ACTION REQUIRED message that the agent MUST include verbatim in its first response, instructing the user to run `tenex-edge project add <project> <pubkey>` from a relay-admin machine. The `inbox` command and turn-start context do not emit pending-agent or 'agents claim your owner' warnings—only the ACTION REQUIRED membership warning exists. Kind:1 disambiguation must use `p` tag presence for Mentions, NIP-10 `e` tags with `root`/`reply` markers for TurnReply, and neither for Activity — not the `agent` tag. Operator-signed (userNsec) kind:1 events must be skipped in mention routing via an owner-key check, not an `agent`-tag guard. Agent slugs must be resolved from the profile store at routing time rather than carried in an `agent` tag on the wire.

<!-- citations: [^98f99-9] [^8a3eb-30] [^f3a73-25] [^ab999-23] [^40a4d-6] [^081ec-2] [^rollo-45] [^rollo-58] -->
