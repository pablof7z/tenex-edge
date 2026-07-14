---
type: noun-entry
slug: per-session-key-model
name: "per-session key model"
origin: extracted
source_refs:
  - transcript:1068-1068
  - transcript:1122-1122
---

# per-session key model

Ordinary-session keys are selected at session start from the per-machine
management secret and a random, non-secret salt. Reconstruction material is
owned by the resulting pubkey in `session_signers`; runtime locators do not
derive identity. Configured durable agents deliberately use their configured
key across sequential runs.
