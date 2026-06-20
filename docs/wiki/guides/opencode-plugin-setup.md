---
title: OpenCode Plugin Setup
slug: opencode-plugin-setup
topic: opencode-integration
summary: The opencode plugin dependency @opencode-ai/plugin must match the installed opencode version (1.16.2) to prevent the plugin from failing to load and opencode fr
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-08
updated: 2026-06-16
verified: 2026-06-08
compiled-from: conversation
sources:
  - session:96aedf14-df2c-425b-b548-0fa7d1c1ba63
  - session:956595fb-fa6a-45f8-869c-b53cae16124f
  - session:55a2eb41-5ff1-4eb3-bdb8-7a4728422be5
  - session:ea5dd578-ca5d-4f31-8427-3a253dd735e8
  - session:9337d29e-ac62-417c-8e99-0cc22cbbfad3
  - session:ses_13089dfceffeDFSl8v4Lv8hCBt
---

# OpenCode Plugin Setup

## Dependency Version

The opencode plugin dependency @opencode-ai/plugin must match the installed opencode version (1.16.2) to prevent the plugin from failing to load and opencode from appearing stuck. <!-- [^96aed-1] -->

The @opencode-ai/plugin@1.16.2 dependency must be installed in both ~/.config/opencode/ and ~/.opencode/. <!-- [^96aed-2] -->

## Plugin Loading

Plugin .ts files for opencode are loaded from ~/.config/opencode/plugin/. Changes to the plugin take effect only on the next opencode launch, not in currently-running sessions. OpenCode keeps its transcript snapshot fresh via the `tool.execute.after` hook. The session-start hook must attempt JSON.parse on its output and extract .session_id (and the `.codename` display label, e.g. `bravo4217`), falling back to treating the output as a bare string (setting the codename to empty) if parsing fails, to handle legacy bare-string responses. The opencode plugin acts as a dumb pipe that passes through the hook's stdout, with no hand-built `selfLine`, `hinted` flag, `run()`, or `stripAnsi()` helpers.

<!-- citations: [^96aed-3] [^95659-1] [^55a2e-1] [^ea5dd-1] [^9337d-1] [^ses_1-25] -->
## Plugin Sources

The tenex-edge.ts plugin is sourced from ~/src/tenex-edge/integrations/opencode/. The proactive-context.ts plugin is sourced from ~/src/proactive-context/integrations/opencode/. <!-- [^96aed-4] -->

## SDK Compatibility

The tenex-edge and proactive-context plugin code is compatible with the @opencode-ai/plugin 1.16.2 SDK without code changes, as info.role, info.id, and info.sessionID are still present in the 1.16.2 types. <!-- [^96aed-5] -->
