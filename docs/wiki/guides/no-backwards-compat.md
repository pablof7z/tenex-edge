---
title: No Backwards Compatibility
slug: no-backwards-compat
topic: repo-discipline
summary: The repository does not preserve backwards compatibility
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-13
updated: 2026-07-13
verified: 2026-07-13
compiled-from: conversation
sources:
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
---

# No Backwards Compatibility

## Policy

The repository does not preserve backwards compatibility. Removed surfaces must be removed completely — no hidden aliases, parser aliases, legacy flags, old subcommands, fallback JSON keys, duplicate MCP/tool names, stale e2e commands, compatibility wrappers, or docs teaching the old form.

<!-- citations: [^019f5-8ad63] -->
