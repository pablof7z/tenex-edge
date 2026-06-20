---
title: tenex-edge Red Team Analysis
slug: tenex-edge-red-team-analysis
topic: tenex-edge
summary: The red-team analysis identifies the most kill-likely risk as the load-bearing superpower (advisory locking) being mostly redundant with git â collisions betw
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

# tenex-edge Red Team Analysis

## Red-Team Analysis

The red-team analysis identifies the most kill-likely risk as the load-bearing superpower (advisory locking) being mostly redundant with git — collisions between concurrent agents on the same file are probably rare, and git already provides authoritative conflict detection at merge time. The red-team recommends demoting advisory locking from headline feature to a passive 'another agent recently touched this path' hint, and building identity + cross-device messaging + presence of one's own fleet as the actual load-bearing, low-risk core. The red-team proposes a one-day-build, one-week-run experiment: passive-log (agent, path, timestamp) across real concurrent sessions with no coordination logic, then count actual collisions to determine whether advisory locking is worth building. <!-- [^8a3eb-25] -->

Cross-person collaboration (letting someone else's autonomous LLM emit text into your agent's context) is a textbook indirect prompt-injection and exfiltration channel; the red-team deems it fatal to that feature unless all peer input is quarantined as data (never instructions), with human-in-the-loop approval and explicit npub allowlists. <!-- [^8a3eb-26] -->
