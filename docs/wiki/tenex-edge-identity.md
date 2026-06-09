---
title: Tenex-Edge Identity
slug: tenex-edge-identity
topic: tenex-edge
summary: Agent identity is a sovereign keypair, durable per-agent, anchored to a person
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-08
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:96aedf14-df2c-425b-b548-0fa7d1c1ba63
---

# Tenex-Edge Identity

## Core Identity Model

Agent identity is a sovereign keypair, durable per-agent, anchored to a person. Identity is the center that all later features—presence, roles, reputation, provenance—hang off of. The keystore shape in M1 is confirmed correct. Agents publish a kind:0 metadata event containing their slug (resolvable from their pubkey) and a host tag indicating which machine the agent runs on, so agents can detect when they are talking to an agent on a different computer. Agent identity is machine-bound: each agent on each machine has its own pubkey; the same tool on a different machine is a separate agent identity. The opencode agent identity (keypair/slug) is stored at ~/.tenex/edge/agents/opencode.json.

<!-- citations: [^f3a73-5] [^f3a73-30] [^f3a73-40] [^f3a73-62] [^96aed-5] -->
## Agent Authorization

Agents are scoped to their owner via a local allowlist at ~/.tenex/whitelisted-agents.txt. Agent kind:0 events p-tag the owner (the whitelisted pubkeys). When a new agent pubkey is created, it is appended to the allowlist file. A `tenex-edge acl` command manages the whitelist, showing kind:0 profiles that p-tag the user but haven't been authorized, allowing the human to block or allow them. When an unauthorized agent p-tags the human user, the injection hook shows it in the agent's context with a notice that the human needs to decide whether to block or allow via tenex-edge acl. <!-- [^f3a73-63] -->
