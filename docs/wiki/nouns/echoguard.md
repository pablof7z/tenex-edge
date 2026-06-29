---
type: noun-entry
slug: echoguard
name: "EchoGuard"
origin: extracted
source_refs:
  - transcript:1304-1306
  - transcript:489-491
---

# EchoGuard

A per-session hash ring (60s TTL) replacing the `[tenex-edge]` text marker for echo suppression. Records what the tmux paste path typed; `rpc_user_prompt` consumes the match to decide not to re-publish daemon-injected envelopes back into the channel.
