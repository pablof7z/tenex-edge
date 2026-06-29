---
title: Tenex-Edge Channel Create
slug: tenex-edge-channel-create
topic: tenex-edge
summary: "`channels create` resolves the parent channel in this precedence: `--parent-channel <ref>`, then the creating agent's current channel (the default), then an exp"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-29
updated: 2026-06-29
verified: 2026-06-29
compiled-from: conversation
sources:
  - session:b20ef4ab-0b54-4770-a549-4ed195c0035e
---

# Tenex-Edge Channel Create

## Parent Channel Resolution

`channels create` resolves the parent channel in this precedence: `--parent-channel <ref>`, then the creating agent's current channel (the default), then an explicit literal `parent` kept for the launch picker, operator invocations, and tests. It no longer sends a cwd-resolved project as the parent. `--parent-channel <ref>` accepts a project-relative reference (e.g. `planning`, `epic999/planning`) resolved within the creator's project subtree, with ambiguity returning candidate paths and exit 2, mirroring `channels switch`. <!-- [^b20ef-fa36b] -->

Creator resolution uses the strict no-project-fallback path so child-of-current and auto-switch only fire when actually run as an agent. <!-- [^b20ef-750ad] -->

## Auto-Switch on Create

The auto-switch on channel create reuses a shared `rehome_session_to_channel` helper extracted so that create and switch share the same route-scope and identity move logic. <!-- [^b20ef-a739c] -->
