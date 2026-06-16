---
title: Tenex Edge
slug: tenex-edge
topic: tenex-edge
summary: tenex-edge owns identity and awareness as its own substrate, independent of any host; pc is a context-injection adapter and its awareness board will be removed
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
---

# Tenex Edge

## Identity & Architecture

tenex-edge owns identity and awareness as its own substrate, independent of any host; pc is a context-injection adapter and its awareness board will be removed once tenex-edge drives awareness. tenex-edge has no concept of any specific host (pc, Claude Code, etc.); it exposes a generic, host-agnostic boundary ('report activity' and 'subscribe to awareness') and the dependency arrow points one direction only (host → tenex-edge). Identity is (agent, machine): each agent is bound to one machine; the same tool on a different machine is a separate agent with a separate pubkey (ideally a different name), and host-on-kind:0 is correct. A per-session process model is used (not a shared daemon). <!-- [^f3a73-3] -->

Local state uses SQLite. Agents publish a kind:0 event with their slug derived from their pubkey; the kind:0 includes a 'host' tag indicating the machine the agent runs on, and p-tags the user's owner pubkey(s) (from whitelistedPubkeys in config). <!-- [^f3a73-4] -->

Trust is managed via an explicit allowlist at ~/.tenex/whitelisted-agents.txt; own-fleet keys are auto-appended when created; agents only see/trust those on the list plus themselves. <!-- [^f3a73-5] -->

## Session Lifecycle

MVP session start is `tenex-edge session-start --agent <agent-slug>` which forks and publishes a presence heartbeat every 30 seconds as NIP-24011 ephemeral events. Session-end stops the background process; the process monitors whether the parent session is still running so it stops publishing heartbeat if the parent dies. Stale peer sessions are pruned: `who` only lists peers with a heartbeat fresher than 90s (3× the 30s tick), and the engine prunes rows older than 10 minutes each tick. <!-- [^f3a73-6] -->

Project slug is derived from .tenex/project.json if present, else the git repo name (shared across worktrees), else the basename of $PWD. Whitelisted pubkeys come from ~/.tenex/config.json 'whitelistedPubkeys'. Relay is configured in ~/.tenex/config.json. <!-- [^f3a73-7] -->

The engine subscribes to kind:0 events p-tagging the owner; foreign agents claiming the owner appear as 'pending' and the injection hook surfaces a notice telling the user to run `tenex-edge acl`. <!-- [^f3a73-8] -->

## Presence & Awareness

Presence heartbeat events use kind:24011 with p-tagged whitelisted pubkeys, d-tagged project slug, agent-tagged pubkey+slug, and session-id tag. Agent awareness is published as kind:1 with a 't' tag of the project slug, describing what the agent is doing. Agents maintain a running NIP-38 status d-tagging their project slug (empty when idle). The UserPromptSubmit hook injects the list of reachable agents (from `who`) plus their current activity/status and any pending mentions into the agent's context each turn. <!-- [^f3a73-9] -->

## Mentions

Mentions (not 'direct messages') are sent via `tenex-edge send-message agentSlug@projectSlug <message>` as kind:1 with t-tagged project and p-tagged destination agent's pubkey. Mentions can target a specific session via --recipient <session-id>, which is important when the same agent runs in multiple sessions simultaneously. Mentions are deduped per-agent (not per-session); once an agent has seen a mention in any session, it is never re-delivered in a later session. A send-message skill is available so Claude Code and other harnesses can send messages to other agents. <!-- [^f3a73-10] -->

## Transport & Codec

Envelope shapes are decoupled from business logic via a modularized encoder/decoder codec; kind:1 is just the initial shape adapter, and future transports (NIP-29 community, Marmot/MLS) can be added as additional shape adapters. Heartbeat activity is part of the codec since it will vary for different codecs. The transport layer uses nostr-sdk (not NMP) behind a codec seam; NMP is a full app kernel unsuitable for headless CLI embedding, and remains a documented future swap-in behind the transport trait. <!-- [^f3a73-11] -->

Transport forces NIP-42 AUTH completion (via a warm-up fetch) before any subscribe, because relay.tenex.chat silently closes subscriptions opened before auth completes. nostr EventBuilder must use allow_self_tagging() for mentions, profiles, and presence p-tags, because nostr strips p-tags equal to the author's own pubkey by default, breaking same-agent cross-session messaging. <!-- [^f3a73-12] -->

rustls CryptoProvider must be explicitly installed at process startup (ring) because rig-core's reqwest pulls aws-lc-rs alongside nostr-sdk's ring, and rustls 0.23 refuses to auto-pick a default when both are present. On macOS, xattr -cr + codesign --force --sign - must be run after copying the binary to ~/.local/bin/, otherwise macOS SIGKILLs the binary on its self-re-exec (fork path). <!-- [^f3a73-13] -->

## Distillation

Auto-distillation (not manual) is used for agent activity; it works like pc with an LLM-based distiller reading the conversation transcript (not just tool names). Claude Code and Codex use the file transcript_path; OpenCode uses its SDK message store written to a temp JSONL. Distillation config lives in ~/.tenex/ using the existing providers.json + llms.json format; the 'edge-distillation' role selects the named model, and the LLM call is made natively via rig-core supporting openrouter and ollama. The distiller ordering is: $TENEX_EDGE_DISTILL_CMD override → edge-distillation role via rig → heuristic fallback. <!-- [^f3a73-14] -->

## Observability

`tenex-edge tail -f <optional-project-slug>` provides a colorized client streaming all messages. Injection of peer messages into agent context is in M1 scope. <!-- [^f3a73-15] -->
