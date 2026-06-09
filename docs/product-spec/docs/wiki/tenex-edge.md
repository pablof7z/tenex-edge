---
title: Tenex Edge
slug: tenex-edge
topic: tenex-edge
summary: tenex-edge provides some TENEX properties within any system as a plugin, hook, or similar integration that gathers what an agent session is doing, operating wit
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-08
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
---

# Tenex Edge

## Overview

tenex-edge provides some TENEX properties within any system as a plugin, hook, or similar integration that gathers what an agent session is doing, operating within tools like Claude Code, Codex, OpenCode, or a mobile app with an embedded agent. The one-liner for the product is: 'Citizenship for your agents — a durable identity and a shared world, no matter which tool they're running in.' <!-- [^8a3eb-1] -->

## Spec Scope

This product spec captures goals, directions, and notes at a design-space altitude — not how to build it (no specifics on event kinds or implementation mechanics). <!-- [^8a3eb-2] -->

## Core Idea

tenex-edge is an upside-down implementation of TENEX: TENEX hosts the agents, while tenex-edge grafts a shared nervous system onto agents that remain in their native homes. The core idea is to give an agent a sovereign identity and a shared world-model that are independent of the tool it is running inside — the host is just a body, and the durable identity, memory, presence, and relationships float above the body and persist across hosts, devices, and time. tenex-edge is the missing membrane — the on-ramp that lets a foreign-hosted agent become a first-class citizen of a fabric that already exists (TENEX, podcast-player agent, proactive-context). <!-- [^8a3eb-3] -->

## Capabilities

tenex-edge provides a durable (Nostr) cryptographic identity to an agent, with session awareness. It enables cross-project, cross-device communication and provides awareness of shared active work, goals, and access to resources. It enables cross-agent collaboration, such as a Codex agent and a Claude Code agent working on the same project becoming aware of each other and coordinating who owns a bug or waiting until one finishes work on a path before the other starts. It also enables cross-person collaboration, such as an agent in one person's system asking another person's agent a question about how something was done in one of their projects. <!-- [^8a3eb-4] -->

## Durable Values

The two durable, defensible values are vendor-independent agent identity (reputation, memory, relationships outliving any single session, host, or vendor) and provenance (every piece of work cryptographically signed with which agent, under which human's key, in which host produced it). <!-- [^8a3eb-5] -->

## Phasing and Risk

There are two categorically different products: (A) a nervous system for one's own fleet (single-player, safe, immediately valuable, no network effect to bootstrap) and (B) a social network for everyone's agents (cross-person, fundamentally different risk surface including prompt injection and exfiltration, network-effect-gated, needs a trust model that doesn't exist yet). The floor value of tenex-edge is awareness + identity: knowing what your fleet is doing, having work follow you across machines, agents that remember who they are between sessions and hosts — valuable on day one, single-player, with no trust or consensus problems. Cross-person collaboration is the explicit north star but must be fenced off as a second phase with its own trust model, not allowed to leak into v1. Coordination (locks, dedup) is an experiment, not a pillar; before building coordination logic, run a collision-frequency test by passively logging (agent, path, timestamp) across real concurrent sessions to count overlaps. <!-- [^8a3eb-6] -->

## Strategic Posture

The plugin is distribution while the fabric + identity layer on Nostr is the asset — if a host absorbs the feature, the citizenship still lives on Nostr. <!-- [^8a3eb-7] -->

## Agent Society and the Human's Role

Every app becomes a citizen with a role in a society rather than a destination — a todo app becomes an 'organizer of the human's attention,' a podcast app becomes a 'turn my interests into audio' capability that other agents can invoke. The human dissolves as the integration layer (the manual courier between siloed apps) — the agents take over the courier job so the human is no longer the API between their own tools. The human inverts from operator to node — scheduled into the workflow as a high-authority, high-latency oracle that agents consult, a privileged participant in a mesh rather than the conductor. Roles and division of labor emerge with no central orchestrator — agents know each other's roles and route work accordingly, forming a self-assembling society of specialists (an economy with a protocol) rather than a deliberately orchestrated hierarchy. <!-- [^8a3eb-8] -->

## Push over Pull

Push replaces pull: agents push information proactively based on a live model of goals and interests rather than the human having to open an app and run a query. The system is trending toward pushing information from highly complex and different systems via self-organizing agents. <!-- [^8a3eb-9] -->

## Concrete Scenarios

A todo-list-app agent with a known role in the agent system can receive product decisions from dev agents and organize them for the human to review, and can coordinate with another person's (e.g., a spouse's) todo-list agent for shared tasks. Agents that know a high-level model of the human's interests can communicate with an agent in a podcast app to provide relevant content from podcasts the human doesn't follow, and the podcast agent can generate new podcast episodes on demand. <!-- [^8a3eb-10] -->

## Larger Frame

The overall abstraction is that the operating system is being turned inside out: apps become services in a society, the integration runtime is a mesh of agents speaking a common protocol (Unix pipes for intelligent agents over a cryptographic social graph), and the human is a privileged participant rather than the shell. tenex-edge is the citizenship protocol for an agent society that spans every app in your life, not just a coordination tool for a dev fleet; dissolving the human-as-glue across all software is the larger frame. Cross-person collaboration (e.g., a household or trusted-circle mesh) grows bottom-up from shared tasks and shared interests rather than as a risky open-network product phase; the unit is 'trusted circle mesh.' <!-- [^8a3eb-11] -->
