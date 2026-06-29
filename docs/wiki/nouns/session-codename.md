---
type: noun-entry
slug: session-codename
name: "session_codename"
origin: extracted
source_refs:
  - transcript:1937-1947
---

# session_codename

A synthetic per-session display/addressing token: NATO-word + 4-digit hash derived from session_id (e.g. bravo4217). Wired into SessionId's Display impl so every {} format of a session_id renders the codename, never the raw id. Targeted for complete deletion as a product concept (issue #99).
