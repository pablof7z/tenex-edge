---
type: noun-entry
slug: x-openai-session
name: "X-Openai-Session"
origin: extracted
source_refs:
  - transcript:801-801
---

# X-Openai-Session

A stable per-conversation header ChatGPT sends unprompted on every request (including `initialize` and every `tools/call`), different per conversation, that can serve as ChatGPT's own session identifier for identity correlation without requiring the client to echo a server-minted header.
