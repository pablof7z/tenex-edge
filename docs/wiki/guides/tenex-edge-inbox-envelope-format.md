---
title: tenex-edge Inbox Envelope Format
slug: tenex-edge-inbox-envelope-format
topic: tenex-edge
summary: Inbox messages are displayed in an email-like envelope format with From, Date, Branch, ID, a '--' separator, and the message body
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-12
updated: 2026-06-16
verified: 2026-06-12
compiled-from: conversation
sources:
  - session:cd74a605-9f83-4e21-a885-4d900e88ce07
  - session:3a3dec25-db3a-4650-8b73-06f0c1687036
  - session:rollout-2026-06-16T14-02-11-019ed018-926e-7c40-bf14-796efbec0b7a
---

# tenex-edge Inbox Envelope Format

## Envelope Structure

Inbox messages are displayed in an email-like envelope format with From, Date, Branch, ID, a '--' separator, and the message body. The envelope separator is fixed at two dashes ('--'). Every inbox message includes a unique ID that the receiving agent can use to reply to that specific message. <!-- [^cd74a-3] -->

The From line includes the sender's session codename in the format `[session <codename>]` (e.g. `[session bravo4217]`). The codename (NATO phonetic word + 4-digit number) is a display convenience, not identity. If the agent is a remote agent, the From line also includes the host in the format `[remote: <host>]`. <!-- [^cd74a-4] -->

The Date field uses the event's actual publish timestamp (from `event.created_at` on the relay-fetch path), not the receipt time. Relative time displays 'just now' for timestamps under 1 minute old. <!-- [^cd74a-5] -->

The envelope captures and displays the sender's workspace state (branch, commit, dirty files) at the time of sending. The dirty file count label uses the singular '1 file dirty' or plural 'N files dirty'. The dirty file count is omitted entirely when the working tree has no modified/untracked files outside .gitignore. <!-- [^cd74a-6] -->


Messages sent via `tenex-edge inbox send` are encoded through the kind1 codec before relay delivery. Every message sent via `inbox send` receives a NIP-29 `h` tag with the project slug as its value. The `h` tag value uses the recipient's project, not the sender's current working directory project. Every message sent via `inbox send` also receives a `p` tag with the recipient's pubkey, which causes it to decode as a `Mention` rather than an Activity. <!-- [^3a3de-1] -->
## Rendering and Data Capture

The same envelope format is used for all mention displays (mid-turn turn-injection, wait-for-mention, and the inbox command), establishing a single unified format. The `format_envelope` function is the single renderer for both the CLI and daemon-side turn injection, ensuring byte-identical output in all contexts. <!-- [^cd74a-7] -->

Sender workspace metadata (branch, commit, dirty files) requires a schema migration to add new columns to the `inbox` table and must be captured in the send path, not just as a display change. <!-- [^cd74a-8] -->

Messages sent via `inbox reply` and the owner-prompt path share the same codec and `h`-tag guarantees as `inbox send`. <!-- [^3a3de-2] -->

Delivered inbox/chat rows are marked before prompt submission so that turn-start hooks do not duplicate them. Exact-row delivery marking must not consume unrelated unread rows. <!-- [^rollo-70] -->
