---
type: research-record
date: 2026-06-26
session: e07193a3-07de-4b5f-81f1-2623b055b20e
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/e07193a3-07de-4b5f-81f1-2623b055b20e.jsonl
source_lines: 626-635
agent_attribution: codex
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Updated design document for durable ordinal agent identities integrated with subscription consolidation, showing identity model architecture and code grounding"
captured_at: 2026-06-26T11:20:04Z
---

Updated design document for durable ordinal agent identities integrated with subscription consolidation, showing identity model architecture and code grounding

---

13669	# Updated Design: Durable Ordinal Agent Identities and Entity Subscriptions
13670	
13671	This supersedes the prior `subscription-design.md`. The key correction is that session-derived transient signers go away. Multiple concurrent instances of the same agent are represented by bounded, deterministic, durable ordinal identities: `smith`, `smith1`, `smith2`, etc.
13672	
13673	## 1. Identity Model
13674	
13675	### Current model being replaced
13676	
13677	Durable agent keys are file-backed at `~/.tenex-edge/agents/<slug>.json` via `identity::load_or_create` (`src/identity.rs:93`) and listed by `list_local_pubkeys` / `list_local_agent_details` (`src/identity.rs:132`, `src/identity.rs:185`).
13678	
