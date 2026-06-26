---
type: research-record
date: 2026-06-26
session: e07193a3-07de-4b5f-81f1-2623b055b20e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e07193a3-07de-4b5f-81f1-2623b055b20e.jsonl
source_lines: 544-549
agent_attribution: codex
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Design validation report: verdict that subscription consolidation approach is sound, but #p set must include durable ordinal identities not singletons"
captured_at: 2026-06-26T11:20:04Z
---

Design validation report: verdict that subscription consolidation approach is sound, but #p set must include durable ordinal identities not singletons

---

15567	**Design: Entity-Based Relay Subscription Consolidation For `tenex-edge`**
15568	
15569	**1. Validation**
15570	
15571	Verdict: the direction is sound, but the proposed `#p` set must not be “durable pubkeys only” as the code stands. Current `tenex-edge` has durable agent keys by default, but duplicate live sessions can use deterministic transient session keys. Those transient pubkeys are used as event authors and can be used as `p` targets.
15572	
