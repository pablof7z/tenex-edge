---
title: OpenCode Configuration
slug: opencode-configuration
topic: tenex-edge
summary: The @opencode-ai/plugin dependency version must match the opencode binary version (1.16.2) in both ~/.config/opencode/package.json and ~/.opencode/package.json.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-09
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:96aedf14-df2c-425b-b548-0fa7d1c1ba63
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
---

# OpenCode Configuration

## Dependency Version

The @opencode-ai/plugin dependency version must match the opencode binary version (1.16.2) in both ~/.config/opencode/package.json and ~/.opencode/package.json. <!-- [^96aed-6] -->

The opencode binary is located at ~/.opencode/bin/opencode. <!-- [^96aed-7] -->

OpenCode's `session.idle` hook must be verified to fire per-turn rather than mid-loop to prevent premature idle states during long turns. <!-- [^95659-3] -->
