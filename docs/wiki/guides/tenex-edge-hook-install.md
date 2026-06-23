---
title: Tenex-Edge Hook Install
slug: tenex-edge-hook-install
topic: tenex-edge
summary: tenex-edge provides an install command that sets up hooks in different harnesses, modeled after proactive-context's install command
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-16
updated: 2026-06-16
verified: 2026-06-16
compiled-from: conversation
sources:
  - session:404ab754-a2d5-4820-a800-6de4972c549c
---

# Tenex-Edge Hook Install

## Install Command

tenex-edge provides an install command that sets up hooks in different harnesses, modeled after proactive-context's install command. The install command supports a --harness flag to target a specific harness, a --all flag to install for all harnesses, a --dry-run flag for preview, a --status flag to check current state, and an --uninstall flag for removal, mirroring the pc install interface.

The install logic lives in a self-contained src/cli/install.rs module. An Install variant exists in the Cmd enum in cli.rs with its handler wired in run(). <!-- [^404ab-7] -->

<!-- citations: [^404ab-1] [^404ab-6] -->
## Claude Code Integration

For Claude Code, the install command JSON-merges 4 hooks (SessionStart, UserPromptSubmit, PostToolUse, Stop) and a statusLine into ~/.claude/settings.json. <!-- [^404ab-2] -->

## Codex Integration

For Codex, the install command JSON-merges 3 hooks (SessionStart with startup|resume matcher, UserPromptSubmit, Stop) into ~/.codex/hooks.json. <!-- [^404ab-3] -->

## opencode Integration

For opencode, the install command drops integrations/opencode/tenex-edge.ts (embedded via include_str!) into ~/.config/opencode/plugin/. <!-- [^404ab-4] -->

## Hook Deduplication

Hook deduplication during install is based on hook signature (hook --host X --type Y) rather than binary path, so reinstalling after a path change replaces existing hooks instead of accumulating duplicates. During install, any existing statusLine that matches the project's own is removed to avoid duplication. <!-- [^404ab-5] -->
