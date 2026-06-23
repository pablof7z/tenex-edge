---
title: the complete via rig distillation call must be
slug: the-complete-via-rig-distillation-call-must-be
topic: general
summary: The `complete_via_rig` distillation call runs asynchronously with a 20-second timeout so it does not block the engine loop
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-16
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:215d979a-a054-4e2b-b349-851e0d874d6d
  - session:633f8f7f-37f8-409c-90a9-ef64b0dc3216
  - session:1b868736-ed6b-4f88-84d9-26bb320accfd
---

# the complete via rig distillation call must be

## Timeout Requirement

The `complete_via_rig` distillation call runs asynchronously with a 20-second timeout so it does not block the engine loop. It returns `Result<Option<String>, String>`, capturing actual rig error messages instead of swallowing them silently.

The turn_first default for scheduling the distiller is 3 seconds (reduced from 30 seconds) so that the first obs loop tick (~5s interval) successfully schedules the distill before most turns end. <!-- [^1b868-11] -->

<!-- citations: [^215d9-3] [^215d9-12] [^633f8-8] -->
