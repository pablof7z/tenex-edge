---
title: tenex-edge Daemon Rebuild and Restart
slug: tenex-edge-daemon-rebuild
topic: architecture
summary: After pushing source changes to origin/master, the locally installed daemon binary is rebuilt and restarted so live hooks/RPCs run the new code
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-14
updated: 2026-06-17
verified: 2026-06-14
compiled-from: conversation
sources:
  - session:rollout-2026-06-14T13-19-49-019ec5a5-1119-76f0-a7e3-36bc985a31bd
  - session:rollout-2026-06-17T10-15-05-019ed46f-0289-7cf3-ae87-5a65210ee266
---

# tenex-edge Daemon Rebuild and Restart

## Rebuild and Restart Workflow

After pushing source changes to origin/master, the locally installed daemon binary is rebuilt and restarted so live hooks/RPCs run the new code. Turn-start context is assembled inside the daemon RPC, so the installed binary and running daemon both need updating for opencode to see renderer changes.

<!-- citations: [^rollo-50] [^rollo-91] -->
