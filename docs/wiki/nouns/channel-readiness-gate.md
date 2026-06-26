---
type: noun-entry
slug: channel-readiness-gate
name: "channel readiness gate"
origin: extracted
source_refs:
  - transcript:877-887
---

# channel readiness gate

idempotent `ensure_channel_ready(ctx: ChannelCtx)` method on `Nip29Provider` in `src/fabric/nip29/readiness.rs` that all three domain publish methods (`publish`, `publish_checked`, `set_status`) converge on; uses TTL-cached fast path, per-channel single-flight mutex, local SQLite read-model checks, and recursive parent ensures before provisioning a channel
