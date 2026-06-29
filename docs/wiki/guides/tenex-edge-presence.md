---
title: Tenex-Edge Presence
slug: tenex-edge-presence
topic: tenex-edge
summary: "Agent liveness in channels is communicated solely via kind:30315 status event TTL (NIP-40 expiration)."
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:3c769f4a-9947-4d7b-a8f5-58355620b951
  - session:bd8689c8-4a5f-45b3-9dbe-758baec2a2f4
---

# Tenex-Edge Presence

## Agent Liveness

Agent liveness in channels is communicated solely via kind:30315 status event TTL (NIP-40 expiration). <!-- [^3c769-8e749] -->


The `nak_relay_observes_transient_duplicate_status_author` integration test currently fails because zero kind:30315 statuses reach the test relay, a pre-existing status-delivery issue independent of the identity split. <!-- [^bd868-cb3cd] -->
## Agent-Context Fabric Output

The agent-context fabric output includes a self-header line showing the caller's agent label, project/channel, host, pubkey, status, member, and pending counts. <!-- [^bd868-f4d1e] -->

## Chat Mention Confirmation

CLI confirmation for chat mentions prints the resolved agent label (e.g. mentioning @haiku1) from the mentioned pubkey instead of the codename. <!-- [^bd868-d64da] -->
