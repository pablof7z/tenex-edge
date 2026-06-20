---
type: research-record
date: 2026-06-18
session: 52474db7-1e81-4011-a859-6343bfeae807
transcript: /Users/pablofernandez/.claude/projects/-Users-pablofernandez-src-tenex-edge/52474db7-1e81-4011-a859-6343bfeae807.jsonl
source_lines: 326-441
agent_attribution: main
has_preregistered_criteria: true
has_method: true
has_structured_report: true
characterization: "Experiment comparing session title generation with vs without tool_use in distillation context, finding that stripping tool_use makes titles anchor on user intent instead of agent actions"
captured_at: 2026-06-18T00:51:28Z
---

Experiment comparing session title generation with vs without tool_use in distillation context, finding that stripping tool_use makes titles anchor on user intent instead of agent actions

---

Assistant: Rather than hunt for the exact session, I'll construct the problematic transcript synthetically (user says "episodes + explicit user messages", agent immediately starts reading Rust) and iterate against the real model.

Let me find the API key first:

User: [Tool result]: agents
cli
config.json
daemon
data
edge
embed.json
embeddings.db
home
identity-cache.db
identity-cache.db-shm
identity-cache.db-wal
identity.pid
identity.sock
llms.json
llms.json.bak-edge
projects
providers.json
runtime
skills
state.db
summarizer
whitelisted-agents.txt

User: [Tool result]: {
  "providers": {
    "openrouter": {
      "apiKey": "${OPENROUTER_KEY}"
    },
    "anthropic": {
      "apiKey": [
        "REDACTED",
        "REDACTED"
      ]
    },
    "codex": {
      "apiKey": "none"
    },
    "ollama": {
      "apiKey": "http://localhost:11434"
    }
  }
}

Assistant: Now I'll write the experiment script:

User: [Tool result]: File created successfully at: /Users/pablofernandez/src/tenex-edge/scripts/distill_experiment.py (file state is current in your context — no need to Read it back)

User: [Tool result]: Model: openai/gpt-4o-mini
Testing current prompt against problem scenarios...


============================================================
SCENARIO: A: old behavior (tool_use in transcript)
INPUT:
  User: I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)
  Assistant: [uses Read src/codec/kind1.rs] [uses Bash grep -rn "episode" src/] [uses Read src/codec/kind1/groups.rs] [uses Bash grep -rn "struct.*Episode" src/]
OUTPUT:
  TITLE: Modify episode structure to include user messages  
  NOW: analyzing code for episode structure

============================================================
SCENARIO: B: new behavior (tool_use stripped)
INPUT:
  User: I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bun

Assistant: Results are better than expected already! Scenario A even with tool-use gives a decent title. Let me try harder cases that better match the real failure — generic grep calls and raw user input with the `❯` prefix the harness injects:

User: [Tool result]: The file /Users/pablofernandez/src/tenex-edge/scripts/distill_experiment.py has been updated successfully. (file state is current in your context — no need to Read it back)

User: [Tool result]: OUTPUT:
  TITLE: Modify episode structure to include user messages  
  NOW: updating episode codec for user messages

============================================================
SCENARIO: D: nudge-to-keep (title already correct)
INPUT:
  CURRENT TITLE: Store explicit user messages in episodes

  TRANSCRIPT:
  User: ok now make sure the assistant content is truncated to 500 chars max
  Assistant: I'll cap assistant content in the episode serializer.
OUTPUT:
  TITLE: Store explicit user messages in episodes  
  NOW: capping assistant content to 500 characters

============================================================
SCENARIO: E: vague user message
INPUT:
  User: what's wrong with it
  Assistant: Let me check the distillation logs.
OUTPUT:
  TITLE: Analyze distillation logs for issues
  NOW: checking the distillation logs

============================================================
SCENARIO: F: generic tool calls (realistic failure case)
INPUT:
  User: ❯ I want episodes to contain

Assistant: All scenarios now produce good titles. The tool_use stripping is the key fix — every case correctly anchors on the user's stated intent. The model was never fundamentally broken, it just needed cleaner signal.

Here's a summary of findings:

| Scenario | Before (tool_use included) | After (stripped) |
|---|---|---|
| Generic grep calls | "Analyze Rust code for specific structures" ❌ | "Modify episode structure to include user messages" ✓ |
| Agent reads wrong files | "Extract relevant code sections" ❌ | "Include user messages in episodes" ✓ |
| Just user message | good ✓ | good ✓ |
| Nudge-to-keep | good ✓ | good ✓ |

**The prompt itself is fine.** When only tool_use calls are emitted (which happens in the first 3 seconds of a turn), they were the only "assistant content" visible, so the model used them as the title signal instead of the user message. Stripping them means those early turns show just the user message — perfect anchor.
