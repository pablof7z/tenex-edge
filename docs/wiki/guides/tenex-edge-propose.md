---
title: tenex-edge Propose Command
slug: tenex-edge-propose
topic: tenex-edge
summary: "tenex-edge propose publishes a kind:30023 event signed by the agent's identity"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-14
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:ab9998c4-6e65-410e-b298-122a2072171c
  - session:40a4d401-2520-4781-b747-b0ef19594bed
  - session:56f9fe89-5ff7-4e5b-b202-334cd7629d42
  - session:d683a556-03b8-4827-b84d-5395cd3610af
---

# tenex-edge Propose Command

## Proposal Event

tenex-edge propose publishes a kind:30023 event signed by the agent's identity. The --thread argument becomes an NIP-10 ["e", <event-id>, "", "root"] tag linking the proposal to a conversation. The --d argument allows revising an existing proposal at the same naddr; omitting it mints a new one. The command works without a live session: it falls back to the cwd for project and TENEX_EDGE_AGENT / --agent for slug, and omits the session-id tag when no live session exists.

<!-- citations: [^ab999-17] [^40a4d-2] [^56f9f-6] -->
## Owner-Signed Note Admission

Owner-signed notes are admitted via an ownership gate: a kind:1 event with a p tag and session-id but no agent tag, signed by the owner's nsec, is routed to the exact session that authored the proposal and rendered as 'from operator'. <!-- [^ab999-18] -->

## Note and Command Semantics

Notes and 'do it' are plain kind:1 content with no structured tags (no 'approve'/'proposal' machine-tags). Capable agents read natural language and act. The only new tenex-edge piece for M1 is propose plus owner-signed-note admission. <!-- [^ab999-19] -->

## tenex-off as Direct Nostr Client

tenex-off is a direct Nostr client that publishes the kind:1 event itself (signed with the human's nsec, tagged to route to the session), rather than calling a send-message tool. tenex-edge's only work is inbound admission of owner-signed notes. <!-- [^ab999-20] -->

## tenex-off Core Routing Updates

tenex-off core was updated to capture proposal routing tags (project from h tag, session from session-id tag) in ArticleSummary, filter proposals by owner pubkey via #p tag, and stamp published notes with p/h/session-id routing tags when both fields are present. <!-- [^ab999-21] -->

## Publish Verification and Error Surfacing

The root cause of the propose/publish silent data loss bug was that nostr-sdk's send_event/send_event_builder resolve Ok as long as the message was transmitted, but the real per-relay NIP-01 OK verdict lives in output.success/output.failed; the publish paths for propose and doctor only read the optimistic write-side ack, so a NIP-29 relay rejecting the event still reported publish: OK with exit 0. publish_signed_checked now returns the EventId (previously returned ()) and routes through a shared assertion helper (assert_relay_accepted) that fails unless at least one relay is in the success set, surfacing the relay's stated rejection reason or a timeout message when no OK ever arrives. A publish_builder_checked helper exists for the doctor probe's connection-key publish. provider.publish_checked() is used by propose so that a relay rejection becomes a hard error and the CLI exits nonzero with the reason. doctor_probe uses the checked publish so its publish: line reflects the true relay verdict instead of a false OK. After publishing, rpc_propose reads the event back by id (reads are open on relay29's closed+public groups) and returns a retrievable field in the response. The CLI prints a loud warning when the relay ACKed but the event is not retrievable on read-back. Unit tests for assert_relay_accepted cover accept, reject-with-reason, and silent-timeout scenarios. <!-- [^d683a-4] -->
