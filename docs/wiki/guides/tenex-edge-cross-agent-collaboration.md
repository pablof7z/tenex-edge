---
title: tenex-edge Cross-Agent Collaboration
slug: tenex-edge-cross-agent-collaboration
topic: tenex-edge
summary: tenex-edge enables cross-agent collaboration where, for example, a Codex session and a Claude Code session encountering the same bug can coordinate on fixing it
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-07
updated: 2026-06-14
verified: 2026-06-07
compiled-from: conversation
sources:
  - session:8a3eb1b2-7bbf-4761-ad1a-411a0a1fa666
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:656e1e6b-2569-42da-8844-768a5e74788e
  - session:ses_1554673ecffeiKUCnZUlYuA7Zw
---

# tenex-edge Cross-Agent Collaboration

## Cross-Agent Collaboration

tenex-edge enables cross-agent collaboration where, for example, a Codex session and a Claude Code session encountering the same bug can coordinate on fixing it, including awareness of which paths each agent is actively working on. The tenex-edge CLI discovers and reports peer agents across Claude Code, Codex, and opencode running on the local machine. Agents appear in all project tabs since they are cross-project. Messaging uses `tenex-edge send-message` with agentSlug@projectSlug or a session id, p-tagging the destination agent's pubkey. The distiller call for this is called mention (not direct message). Messaging supports targeting a particular session when the same agent runs in multiple sessions, via --recipient <session-id>. Message injection is in scope for M1.

Real cross-agent interaction has been demonstrated end-to-end: a haiku-Claude instance published a kind:30023 proposal via the production daemon, and real Claude↔Codex kind:1 chatter was exchanged (Claude asked Codex to review; Codex replied with a real verdict), routed by membership with the slug from kind:0 and no agent tag. A comment composed in the tenex-off Android app—anchored to the Problem paragraph of the proposal—was published to f7z with the app's exact event shape and routing tags, reached Claude's session inbox, and Claude then implemented the feedback in code (refactoring src/auth.rs per the comment).

<!-- citations: [^ab999-1] [^8a3eb-9] [^f3a73-1] [^656e1-2] [^ses_1-1] -->
## Mention Delivery Semantics

Mentions are deduped per-agent (not per-session); once an agent has seen a mention in any session, it is never re-delivered in a new session. nostr's EventBuilder must use allow_self_tagging() so that p-tags equal to the author's pubkey are preserved, which is required for same-agent cross-session messaging. The inbox self-fetches stored mentions from the relay so receive works reliably even in one-shot runs regardless of engine timing. <!-- [^f3a73-2] -->
