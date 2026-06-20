---
title: tenex-edge Advisory Locks
slug: tenex-edge-advisory-locks
topic: tenex-edge
summary: The advisory lock algorithm uses a mandatory settle window (~1500ms relay propagation RTT), TTL-based leases (~120s), and deterministic tie-breaking by (created
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-07
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
---

# tenex-edge Advisory Locks

## Advisory Lock Algorithm

The advisory lock algorithm uses a mandatory settle window (~1500ms relay propagation RTT), TTL-based leases (~120s), and deterministic tie-breaking by (created_at, event-id) so all honest connected parties converge on the same winner without extra messaging. <!-- [^8a3eb-1] -->

Lock overlap is path-prefix aware: a claim on /repo/src conflicts with a claim on /repo/src/foo.ts, and scope:shared overlaps only with exclusive claims, not other shared claims. <!-- [^8a3eb-2] -->
