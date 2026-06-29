---
type: noun-entry
slug: daemon
name: "daemon"
origin: extracted
source_refs:
  - transcript:78-85
  - transcript:135-144
---

# daemon

ONE daemon per machine is the sole owner of state.db, the single relay connection, the inbox, presence, membership cache, and peer pruning; every CLI invocation and every per-session engine becomes a thin client that talks to it over a Unix domain socket.
