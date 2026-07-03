---
type: noun-entry
slug: role-tenex
name: "role (TENEX)"
origin: extracted
source_refs:
  - transcript:15-26
  - transcript:971-971
---

# role (TENEX)

A named key in llms.json that code resolves to a concrete model + credentials; the canonical example is edge-distillation. Resolution path: role → llms.json[role] (config name) → configurations[name] → {provider, model} → apiKey from providers.json.
