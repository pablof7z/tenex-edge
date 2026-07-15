---
title: Mosaico Agent Identity
slug: mosaico-agent-identity
topic: mosaico
summary: Pubkeys are authoritative; handles are public aliases
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-07-14
verified: 2026-07-14
compiled-from: conversation
sources:
  - session:019f12ce-2569-72e0-b959-6d87d5daec5d
  - session:bd8689c8-4a5f-45b3-9dbe-758baec2a2f4
  - session:019f12f9-8a0b-7012-ad2f-f4d0cb035d2b
  - session:75f62bb9-f564-4633-8741-997dfea1d0e7
  - session:4d65680c-ded1-47cd-a59a-4966eebe8eda
---

# Mosaico Agent Identity

## Pubkey Authority

The Nostr pubkey is the authoritative session identity. Ordinary sessions derive
their signer from the machine management secret and a random, non-secret salt.
The salt is stored once in `session_signers`, keyed by the resulting pubkey. A
runtime row id or harness resume token never participates in key derivation.

Agents configured with `perSessionKey:false` are the explicit durable exception:
they sign with the key in `<mosaico_home>/agents/<slug>.json` and may reuse that
pubkey across sequential runs, with at most one active run at a time.

Ordinary `perSessionKey:true` agents store no secret or public key in agent JSON.
Existing redundant key fields are removed when such a config is loaded. Native
harness profiles discovered without a Mosaico agent JSON use this ordinary
per-session identity path as well.

## Agent/Session Handle

Each ordinary session publishes a kind:0 profile with a leased public handle.
For example, a Codex session can be mentioned as `@quill-codex`. The handle is a
human-facing alias mapped to the pubkey; it is not another signing identity.

## Trust Is Channel Membership

Trust is NIP-29 channel membership, exclusively. The machine's management key adds
a session's pubkey as a member of a channel; a session is removed from membership
on clean end and after 10 minutes with no heartbeat (TTL prune). An expired
session still appears in `who --expired` and remains re-derivable and resumable —
membership is presence, not the definition of the session's identity.

## Roster vs. Members

The roster (`available-agents`) is the set of explicit role configs plus valid
native Codex, Claude Code, and OpenCode profiles installed on the machine or in
the bound workspace — the *types* you can add to a channel. Channel *members* are concrete sessions,
rendered by their dashed public handles. Adding a role to a channel spawns a
new session; that session is what becomes a member.

## Session Identification and Routing

The pubkey signs events and is the routing identity. The leased handle resolves
to that pubkey for human-facing selection. Runtime ids, native harness tokens,
PTY endpoints, sockets, and PIDs are local correlation or transport locators.
They do not derive, replace, or alias the signing identity.

`resolve_session_inner` is the central daemon function for resolving an exact
caller identity. `my session` and its status mutation enter through the strict
`resolve_caller` wrapper, so a self-scoped command cannot bind an arbitrary
sibling session. Human `who` does not resolve an agent session.

<!-- citations: [^4d656-e8fdc] -->
## Session Resume

Resume reconstructs an ordinary signer from the pubkey-owned salt and the
machine management key. The leased handle remains mapped to that pubkey, so a
resumed session preserves both its signing identity and public name without
using a runtime locator as cryptographic input.

## Identity Commands

`mosaico my session` shows the caller who they are and which fabric they
inhabit: an XML `<self>` row followed by global agent capabilities and
workspace/channel membership. Every workspace joined by that exact session is
expanded, while merely known workspaces remain compact. Human `who` is a
separate terminal-only operator projection. The caller member match keys on the
session's pubkey.
