---
title: Tenex-Edge Hook Output Rendering
slug: tenex-edge-hook-output-rendering
topic: tenex-edge
summary: Hook warnings marked as BLOCKING should be framed as prerequisites to answering, making them harder for the assistant to skip
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-15
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:1562957b-67e8-4ac1-a48b-84e8ec1696bb
  - session:a0037729-ad51-460a-880d-0a9699f6ee41
---

# Tenex-Edge Hook Output Rendering

## Warning Placement and Framing

Hook warnings marked as BLOCKING should be framed as prerequisites to answering, making them harder for the assistant to skip. Hook warnings should be placed before general instructions in the output, not buried after them, to ensure they are not overlooked. Hook output containing a direct instruction to tell the user something (e.g., a blocking warning) must be relayed to the user rather than ignored. Hook instructions phrased as a prerequisite (e.g., 'Before responding, tell the user…') or marked with 'BLOCKING:' are more reliably acted upon than those labeled 'WARNING:'. Hook output containing instructions like 'Tell the user to run the following command' should be treated as a blocking prerequisite that must be surfaced before responding to the user.

<!-- citations: [^15629-31] [^15629-37] [^15629-38] [^15629-46] [^15629-58] -->

## Hook Output Envelope and Routing

Claude Code PostToolUse only reads context from the `hookSpecificOutput.additionalContext` JSON envelope; plain stdout is ignored. The hook must emit JSON in that envelope format with exit 0. The `EmitFormat` enum selects the correct output envelope per hook type: `hookSpecificOutput` for Claude Code PostToolUse, plain text for UserPromptSubmit/OpenCode, and `systemMessage` for Codex.

<!-- citations: [^a0037-2] [^a0037-4] -->
