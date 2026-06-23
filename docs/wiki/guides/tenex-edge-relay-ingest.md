---
title: Tenex-Edge Relay Ingest
slug: tenex-edge-relay-ingest
topic: tenex-edge
summary: The `handle_incoming` function deduplicates relay events by event ID using a 512-slot ring buffer (`seen_events`) in `DaemonState` to prevent fanout duplication
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-10
updated: 2026-06-10
verified: 2026-06-10
compiled-from: conversation
sources:
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:36cc4546-228e-4d07-a1a8-9d0cd7cd5a6c
---

# Tenex-Edge Relay Ingest

## Event Deduplication

The `handle_incoming` function deduplicates relay events by event ID using a 512-slot ring buffer (`seen_events`) in `DaemonState` to prevent fanout duplication.

The runtime processes both deduped Event notifications and raw `RelayPoolNotification::Message{Event}` to avoid silent pending-discovery failures, because nostr-sdk suppresses events seen during the warm-up fetch as 'already seen'.

<!-- citations: [^56f9f-7] [^f3a73-119] [^36cc4-8] -->
## Connection Warm-up

Transport::connect forces a warm-up AUTH fetch before subscribing, because relay.tenex.chat silently closes subscriptions opened before NIP-42 AUTH completes. <!-- [^f3a73-120] -->
