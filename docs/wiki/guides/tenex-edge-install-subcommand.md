---
title: tenex-edge Install Subcommand
slug: tenex-edge-install-subcommand
topic: tenex-edge
summary: tenex-edge provides an install subcommand that sets up hooks in the different harnesses, mirroring the proactive-context install command's interface.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-17
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:404ab754-a2d5-4820-a800-6de4972c549c
  - session:rollout-2026-06-17T12-22-51-019ed4e3-fd48-7310-ad79-de5185d87912
---

# tenex-edge Install Subcommand

## Overview

tenex-edge provides an install subcommand that sets up hooks in the different harnesses, mirroring the proactive-context install command's interface. <!-- [^404ab-1] -->

## Flags

The install subcommand provides --harness, --all, --dry-run, --status, and --uninstall flags. <!-- [^404ab-2] -->

## Harness Support

The install command supports Claude Code via JSON merge into ~/.claude/settings.json, wiring 4 hooks (SessionStart, UserPromptSubmit, PostToolUse, Stop) plus a statusLine. It deduplicates any existing statusLine that looks like tenex-edge's during the Claude Code harness setup.

The install command supports Codex via JSON merge into ~/.codex/hooks.json, wiring 3 hooks (SessionStart with startup|resume matcher, UserPromptSubmit, Stop).

The install command supports opencode via file drop of integrations/opencode/tenex-edge.ts into ~/.config/opencode/plugin/, embedding the file via include_str!. <!-- [^404ab-3] -->

## Hook Deduplication

Hook deduplication during install is by hook signature (--host X --type Y) rather than binary path, so reinstalling after a path change replaces hooks instead of accumulating duplicates. <!-- [^404ab-4] -->

## Launch Command

The `tenex-edge launch` command accepts passthrough flags after a `--` separator (e.g. `tenex-edge launch <agent> -- --flag`). Passthrough flags provided to `tenex-edge launch` are forwarded to the harness command. <!-- [^rollo-118] -->
