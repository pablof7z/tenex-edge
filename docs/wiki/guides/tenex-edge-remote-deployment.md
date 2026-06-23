---
title: Tenex-Edge Remote Deployment
slug: tenex-edge-remote-deployment
topic: tenex-edge
summary: The tenex-edge project must be cloned on pablo@157.180.102.242 at ~/Work/tenex-edge/ and configured for use with Claude Code
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-12
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:081ec521-c99b-42fb-9aa7-4a109519a62f
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
---

# Tenex-Edge Remote Deployment

## Remote Deployment

The tenex-edge project must be cloned on pablo@157.180.102.242 at ~/Work/tenex-edge/ and configured for use with Claude Code. The remote machine at pablo@157.180.102.242 must be rebuilt/redeployed after local code changes to get the updated binary. The tenex-edge Rust binary must be installed at ~/.local/bin/tenex-edge (symlinked from the build output) so hook commands can find it. The remote machine's ~/.claude/settings.json merges tenex-edge hooks alongside existing pc hooks without overwriting them. The `.mcp.json` and `.claude/settings.local.json` files created on the remote machine for tenex-edge were incorrect and must be removed, since the integration is hooks-based not MCP-based. On macOS, after copying the binary, run `xattr -cr` and `codesign --force --sign -` on it, or macOS will SIGKILL the binary on the fork/re-exec path due to stale signatures and com.apple.provenance xattr. The production daemon was cut over from the old binary to the refactored worktree binary after verifying migration on a copy of the real state.db (40 projects, 15 members backfilled cleanly), with backups made of both the old binary and the real db.

<!-- citations: [^ab999-91] [^081ec-5] [^081ec-8] [^f3a73-121] -->
