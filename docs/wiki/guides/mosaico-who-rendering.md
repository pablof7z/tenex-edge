---
title: Mosaico Who Rendering
slug: mosaico-who-rendering
topic: mosaico
summary: "The `who` command is human-only: it always renders human-formatted terminal text, never XML, and never mutates the cursor"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-03
updated: 2026-07-14
verified: 2026-07-11
compiled-from: conversation
sources:
  - session:7d6bf2fe-8dc9-4bd0-aeeb-de1827bf68d1
  - session:019f5a74-0a91-7340-8299-8ac3dccfa36d
  - session:019f5fec-4248-78b1-8d8f-8aa1238afb9c
---

# Mosaico Who Rendering

## Audience-Aware Rendering

The `who` command is human-only: it always renders human-formatted terminal text, never XML, and never mutates the cursor. Agent self-awareness is a strict session-scoped read served by `my session`, not `who`; agent invocation of `who` fails with guidance to use `my session`. A bare operator receives terminal-oriented text. Both `who` and `my session` are read projections over the daemon-owned store; neither renderer queries the relay directly. The `who` command is hidden from default agent help but shown in human help; an explicit agent-session invocation succeeds and returns the read-only fabric view.

Every known workspace appears in the human view. The workspace is its root channel, so the workspace row carries root membership directly and only real descendants render as channel rows. By default only the caller's workspace is expanded; `--all-workspaces` expands every workspace the caller has joined. Channel contents recurse only while the caller is a member of each parent channel. Member counts exclude backend keys. Workspace names are deterministically color-coded using a shared console-wide function that derives color from the canonical workspace ID, used consistently across `who`, channel listings, statusline, debug output, and the session picker.

`who` stays a pure human fabric view. Local live-session control belongs in `mosaico sessions`, while `who --expired` owns resumable dead or old sessions. Agent awareness belongs exclusively to `my session`.

<!-- citations: [^7d6bf-cad7a] [^7d6bf-5aab0] [^019f5-43e23] [^019f5-996eb] [^019f5-e2207] -->
