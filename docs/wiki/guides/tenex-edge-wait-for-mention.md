---
title: tenex-edge Wait-for-Mention Command
slug: tenex-edge-wait-for-mention
topic: tenex-edge
summary: The `tenex-edge wait-for-mention` command has been removed from the codebase
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-17
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:3da7f7d8-c5a3-4065-be64-3a3a73dbb1d6
  - session:622711fa-5176-4580-b311-d66446c2924b
  - session:rollout-2026-06-09T13-56-49-019eac07-22a9-7081-8790-0a45cd7a6d93
  - session:rollout-2026-06-17T12-04-57-019ed4d3-96b5-7b73-b076-4969a3d16afa
---

# tenex-edge Wait-for-Mention Command

## Overview

The `tenex-edge wait-for-mention` command has been removed from the codebase. Invoking it now results in an 'unrecognized subcommand' error from the CLI. (Previously: The command blocked until a mention was received, then exited successfully and printed a message.) All related plumbing — including daemon RPC handling, mention notify/waiter logic, tmux armed-waiter logic, first-turn prompt text, docs/wiki references, and the Claude Code channel adapter — has been removed. The codebase contains no remaining references to wait-for-mention, wait_for_mention, or WaitForMention.

<!-- citations: [^3da7f-3] [^rollo-21] [^rollo-117] -->
