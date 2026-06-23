---
title: Tenex-Off Publishing
slug: tenex-off-publishing
topic: tenex-edge
summary: "Tenex-off is a direct Nostr client that publishes kind:1 events signed with the owner's nsec straight to relays; it does not call a send-message tool or route t"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-12
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:ab9998c4-6e65-410e-b298-122a2072171c
---

# Tenex-Off Publishing

## Direct Nostr Publisher

Tenex-off is a direct Nostr client that publishes kind:1 events signed with the owner's nsec straight to relays; it does not call a send-message tool or route through the daemon. Owner-signed notes (kind:1 with p tag + session-id tag, no agent tag, signed by the owner's nsec) are admitted via an ownership gate and delivered to the exact authoring session's inbox as 'from operator' — verified end-to-end on the wire with a real publish from an owner key to nip29.f7z.io routing to a live agent session. The tenex-off Rust core (commit 320fe88) captures proposal routing tags (project from h tag, session from session-id tag) in ArticleSummary, fetches proposals p-tagged to the owner, and stamps published comments with p/h/session-id routing tags when both project and session are present on the article — verified live with an owner-signed note landing in the target session inbox. A real e2e loop was proven on production fabric: an agent publishes a kind:30023 proposal, the phone (owner-signed direct Nostr publish) sends a plain kind:1 note, the daemon admits it via ownership gate and delivers it to the exact authoring session's inbox — no instruction tags, agents choose when to propose. A comment composed and anchored in the tenex-off Android app (published to f7z) was routed to the exact session that authored the proposal, and Claude read the comment and refactored src/auth.rs accordingly. The app's deep-linked document view (OpenNaddr) captures the proposal's project/session routing tags, so comments published from that path carry the h/p/session-id tags needed for delivery. The tenex-off app has no relay-backed conversation feed or thread view yet — networking is behind the nmp_core observer model, and conversation rendering was achieved by seeding the app's local comment store with the real chatter events.

<!-- citations: [^ab999-82] [^ab999-24] [^ab999-45] [^ab999-64] [^ab999-75] [^ab999-81] [^ab999-93] -->
