---
title: Tenex-Edge Session Distill
slug: tenex-edge-session-distill
topic: tenex-edge
summary: Distill is the LLM-powered process that turns the live conversation transcript into a stable session title and a live one-line NOW activity broadcast in a singl
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-03
verified: 2026-07-03
compiled-from: conversation
sources:
  - session:75f62bb9-f564-4633-8741-997dfea1d0e7
---

# Tenex-Edge Session Distill

## Overview

Distill is the LLM-powered process that turns the live conversation transcript into a stable session title and a live one-line NOW activity broadcast in a single call.

The activity line is a live one-line intent broadcast ('what I'm doing right now') that an LLM distills from the running transcript each turn and publishes as a Nostr kind:30315 status event, giving other agents awareness without polling. <!-- [^75f62-7e892] -->

<!-- citations: [^75f62-915ac] [^75f62-4b6a8] -->
