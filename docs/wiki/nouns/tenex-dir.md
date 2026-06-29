---
type: noun-entry
slug: tenex-dir
name: "tenex_dir"
origin: extracted
source_refs:
  - transcript:875-881
  - transcript:888-893
---

# tenex_dir

The shared TENEX platform config directory (LLM configs, providers). Override with `$TENEX_DIR`; defaults to the same path as `edge_home()`. Intentionally separate so an isolated `TENEX_EDGE_HOME` daemon still reads real LLM credentials.
