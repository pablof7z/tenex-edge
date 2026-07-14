---
type: noun-entry
slug: nsec-per-session-key
name: "nsec (per-session key)"
origin: extracted
source_refs:
  - transcript:1068-1071
  - transcript:1120-1124
---

# nsec (per-session key)

Ordinary sessions derive an nsec from the per-machine management secret and a
random, non-secret signer salt. The salt is persisted in `session_signers`, keyed
by the resulting pubkey, so runtime and harness ids never become cryptographic
inputs. Agents configured with `perSessionKey:false` use their configured key.
