---
title: tenex-edge Codex Hook Integration
slug: tenex-edge-codex-hook-integration
topic: tenex-edge
summary: The claude/codex binary hooks use the tenex-edge binary found in $PATH rather than a hardcoded absolute path.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-19
updated: 2026-06-19
verified: 2026-06-19
compiled-from: conversation
sources:
  - session:8fbcc279-f528-4fb3-a2f8-2aec4e9c25aa
---

# tenex-edge Codex Hook Integration

## Binary Hook Path Resolution

The claude/codex binary hooks use the tenex-edge binary found in $PATH rather than a hardcoded absolute path. <!-- [^8fbcc-1] -->
