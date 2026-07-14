---
type: noun-entry
slug: idle-exit-watcher
name: "idle-exit watcher"
origin: extracted
source_refs:
  - transcript:641-651
---

# idle-exit watcher

Background task that shuts the daemon down after it has had no open clients and no live sessions for a configurable grace period (default 120s, overridable via MOSAICO_DAEMON_GRACE_S).
