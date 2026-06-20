---
title: tenex-edge NIP-29 Materializer
slug: tenex-edge-nip29-materializer
topic: tenex-edge
summary: NIP-29 39000/39002 events hydrate state exclusively through Nip29Materializer into store-level materializer methods
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:rollout-2026-06-16T12-40-33-019ecfcd-d47b-7992-998f-75432d8ac4cf
---

# tenex-edge NIP-29 Materializer

## NIP-29 Materialization

NIP-29 39000/39002 events hydrate state exclusively through Nip29Materializer into store-level materializer methods. Store::materialize_membership_snapshot keeps legacy group_members and canonical membership aligned, including revoking stale canonical members absent from the latest relay snapshot. <!-- [^rollo-54] -->

Provider/project-edit/project-add paths publish NIP-29 management events and then refresh relay-authored group state through the materializer instead of directly writing project_meta or group_members. <!-- [^rollo-55] -->

Mention admission and statusline membership read through canonical NIP-29 membership decisions rather than direct legacy table queries. <!-- [^rollo-56] -->
