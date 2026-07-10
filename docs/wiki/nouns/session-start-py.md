---
type: noun-entry
slug: session-start-py
name: "session_start.py"
origin: extracted
source_refs:
  - transcript:697-702
  - transcript:631-638
---

# session_start.py

A single deterministic entrypoint script the agent runs at session start instead of following a static 'list workflows' instruction; it decides what to inject — the setup guide (SETUP.md) when the home dir is not yet tracked in a git repo, or the session brief (tracked location, workflow list, BRIEF.md) when it is.
