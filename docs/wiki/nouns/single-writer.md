---
type: noun-entry
slug: single-writer
name: "single-writer"
origin: extracted
source_refs:
  - transcript:317-318
---

# single-writer

The daemon's architectural property of collapsing N per-session SQLite writers and N relay connections into 1, fixing a real multi-writer corruption class (a genuine incident in the project's git history).
