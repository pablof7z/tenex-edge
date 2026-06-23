---
title: Tenex-Edge Design Philosophy
slug: tenex-edge-design-philosophy
topic: tenex-edge
summary: The design discussion operates at a higher, design-space levelâwhat the thing is, what shape it should take, what is worth wanting, and where tensions and bet
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-09
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:d208c058-7b2b-4ff8-bb82-d63623d51097
---

# Tenex-Edge Design Philosophy

## Design Space Focus

The design discussion operates at a higher, design-space level—what the thing is, what shape it should take, what is worth wanting, and where tensions and bets lie—rather than at the level of specifics such as event kinds or lock algorithms.

There are actually two products here: (A) a nervous system for one's own fleet (single-player, safe, immediately valuable) and (B) a social network for everyone's agents (cross-person, fundamentally different risk surface and adoption model). They must not be conflated. Cross-person collaboration means piping a foreign, autonomous LLM's output into one's own agent's context — a prompt-injection and exfiltration risk, not just a feature with a security footnote. Cross-person is the destination, not the door; it must not leak into v1. <!-- [^8a3eb-27] -->

The product must not require mutual adoption before value appears; single-player value must come first. The floor (definitely real, single-player day-one value) is durable cross-host agent identity + presence/awareness of the user's own fleet; the ceiling (high-wow, high-risk, unproven) is coordination (locks, dedup). Coordination should be an experiment, not a pillar — run a collision-frequency test first (passive-log agent file-touches for a week, count overlaps) before committing to building coordination infrastructure. <!-- [^8a3eb-28] -->

The user's todo-app/podcast-app example abstracts to: apps become citizens (not destinations), the human dissolves as the integration layer between siloed apps, the human inverts from operator to a privileged node in a mesh, roles emerge with no central orchestrator, and push replaces pull. <!-- [^8a3eb-29] -->

The strategic posture is: the plugin is distribution; the fabric + identity layer is the durable asset. If a host absorbs the plugin feature, the citizenship still lives on Nostr. <!-- [^8a3eb-30] -->

A product spec has been written at docs/product-spec/ with 13 chapters covering vision, first principles, problem space, agent society, principles and tenets, scope (two products), value layers, ecosystem, trust and safety, prior art, bets and open questions, roadmap, and glossary. <!-- [^8a3eb-31] -->

<!-- citations: [^d208c-2] [^8a3eb-19] [^8a3eb-20] [^f3a73-10] [^f3a73-11] [^f3a73-12] [^f3a73-39] [^f3a73-50] [^f3a73-60] [^f3a73-81] [^f3a73-88] [^d208c-9] [^8a3eb-26] -->

## Beachhead & MVP

The demo, MVP, and beachhead are the same moment: a solo dev running two heterogeneous agents (Claude Code + Codex) on the same repo, where agents stop clobbering each other and start covering for each other. The beachhead user is the solo agent-power-user running two or more agents on the same repo (specifically the Claude Code + Codex crowd), because their pain is acute and self-inflicted with zero coordination problem to bootstrap. The MVP scope is advisory lock (S1) + shared-bug dedup (S2), solo, across Claude Code and Codex on one machine — strictly single-player with no cross-person features. <!-- [^8a3eb-32] -->

## Roadmap Sequence

The north-star sequence is: Rung 1 (MVP) — solo agents stop fighting (advisory lock + dedup); Rung 2 (v1) — work follows the user across devices, collaborator's agents become visible (read-only feed); Rung 3 (ambitious) — agents and friends' agents form a coordinated mesh with cryptographic provenance. tenex-edge owns identity and awareness as its own independent substrate, with no concept of any specific host (pc, Claude Code, etc.); the dependency arrow points one direction only from hosts to tenex-edge, never the reverse. MCP is the natural shape of the tenex-edge public boundary — a standard interface that any external component integrates against with zero bilateral knowledge. The Rung 0 MVP consists of identity, local awareness, and a passive collision logger that watches but does no coordination, with zero network, zero trust model, and zero consensus; agent routing is the next increment and is not part of v0. Rung 1 lifts the same state onto the fabric (Nostr); this is a transport swap rather than a rewrite, provided the seam is clean. tenex-edge's substrate is testable and valuable with no consumer attached; it can be driven from a test harness or CLI. The implementation language is Rust (not NMP); NMP was found to be a full cross-platform app kernel unsuitable for a headless CLI daemon, so nostr-sdk is used behind the transport trait seam. The architecture requires very clear scoping of concerns and adherence to Single Responsibility Principle (SRP). The architectural proposal document is published as a kind:30023 (NIP-23 long-form) event on the nos.lol relay, signed by the user's `userNsec` identity, with the `d` tag `tenex-edge-fabric-architecture`. <!-- [^8a3eb-33] -->

## Boundary Constraints

The platform bet is a thin open adapter on Nostr — the protocol is the product. tenex-edge owns no agents and no server. No hosted central server is to be built — a coordination server would betray the premise and discard the durable-identity / no-central-server property. No own agent or agent host is to be built — that's TENEX's role, and building one betrays the premise that agents stay in their native homes. No UI-first dashboard / mission control is to be built as the product's center of gravity; lead with hooks and a CLI feed instead. No open-ended agent chat feature is to be built — coordination must be bounded by a shared artifact (a file, a bug, a goal). No mobile app or push backend is to be built in early rungs. <!-- [^8a3eb-34] -->
