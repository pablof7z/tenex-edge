---
type: noun-entry
slug: providers-json
name: "providers.json"
origin: extracted
source_refs:
  - transcript:18-19
  - transcript:79-79
---

# providers.json

A file under mosaico_home() holding provider credentials in the format { "providers": { "<provider>": { "apiKey": ... } } }, where for ollama the apiKey field actually holds the base URL. claude-cli needs no entry in it.
