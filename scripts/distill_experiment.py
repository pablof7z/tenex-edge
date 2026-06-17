#!/usr/bin/env python3
"""
Experiment script: test SESSION_SYSTEM_PROMPT variants against real model.
Usage: python3 scripts/distill_experiment.py
"""
import json, urllib.request, urllib.error, sys, textwrap

OPENROUTER_KEY = "${OPENROUTER_KEY}"
MODEL = "openai/gpt-4o-mini"

# ---------------------------------------------------------------------------
# The current production prompt (from distill.rs)
# ---------------------------------------------------------------------------
CURRENT_PROMPT = (
    "You maintain two labels for a coding session. Output EXACTLY two lines, nothing else:\n\n"
    "TITLE: the session's overall objective — what the agent was asked to accomplish, NOT the step "
    "it happens to be doing right now. A stable noun phrase or imperative, at most 8 words, no "
    "trailing punctuation. Prefer the user's stated request. It must stay valid for the WHOLE "
    "session; if it would go stale in a few messages it is too specific — zoom out to the goal. "
    "You may be given the CURRENT title; if it still fits, repeat it verbatim. Only change it when "
    "the objective itself has substantively changed.\n\n"
    "NOW: what the agent is doing at this moment — the current step or mechanics. At most 8 words, "
    "present tense, no trailing punctuation. This is expected to change every turn.\n\n"
    "Example:\nTITLE: Fix GitHub issue 1\nNOW: reading the issue tracker"
)

# ---------------------------------------------------------------------------
# Test scenarios
# ---------------------------------------------------------------------------

# Scenario A: the failing case — user message + agent tool calls (old behavior, tool_use included)
SCENARIO_A_OLD = """User: I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)
Assistant: [uses Read src/codec/kind1.rs] [uses Bash grep -rn "episode" src/] [uses Read src/codec/kind1/groups.rs] [uses Bash grep -rn "struct.*Episode" src/]"""

# Scenario B: same session, but with tool_use stripped (new behavior)
SCENARIO_B_STRIPPED = """User: I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)"""

# Scenario C: mid-session — agent has replied with text but also did tool calls
SCENARIO_C_MID = """User: I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)
Assistant: I'll look at the episode codec to understand the current structure.
Assistant: I see `Kind1Groups` stores turns. The user message field is currently omitted. I'll add a `user_messages: Vec<String>` field and populate it from the transcript, keeping only the last assistant turn.
User: yes exactly, and for the assistant keep only the last message"""

# Scenario D: nudge-to-keep — title already set correctly, new turn
SCENARIO_D_NUDGE = """CURRENT TITLE: Store explicit user messages in episodes

TRANSCRIPT:
User: ok now make sure the assistant content is truncated to 500 chars max
Assistant: I'll cap assistant content in the episode serializer."""

# Scenario E: tricky — user message is vague, agent action is more informative
SCENARIO_E_VAGUE = """User: what's wrong with it
Assistant: Let me check the distillation logs."""


def call_model(system: str, user: str) -> str:
    payload = json.dumps({
        "model": MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
        "max_tokens": 96,
    }).encode()
    req = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {OPENROUTER_KEY}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            data = json.loads(resp.read())
            return data["choices"][0]["message"]["content"].strip()
    except urllib.error.HTTPError as e:
        return f"ERROR {e.code}: {e.read().decode()[:200]}"


def run(label: str, transcript: str, prompt: str = CURRENT_PROMPT):
    print(f"\n{'='*60}")
    print(f"SCENARIO: {label}")
    print(f"INPUT:\n{textwrap.indent(transcript, '  ')}")
    result = call_model(prompt, transcript)
    print(f"OUTPUT:\n{textwrap.indent(result, '  ')}")


# Scenario F: realistic failure — generic tool calls drown out user intent
SCENARIO_F_GENERIC_TOOLS = """User: ❯ I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)
Assistant: [uses Read src/codec/kind1.rs] [uses Bash grep -rn "struct" src/codec/] [uses Read src/codec/kind1/groups.rs] [uses Bash grep -rn "impl.*Display" src/codec/] [uses Bash grep -rn "pub fn" src/codec/kind1/groups.rs]"""

# Scenario G: even worse — agent opens a totally unrelated file first
SCENARIO_G_WRONG_FILE = """User: ❯ I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)
Assistant: [uses Read src/runtime.rs] [uses Bash grep -rn "pub struct" src/] [uses Read src/state.rs] [uses Bash grep -n "fn distill" src/distill.rs]"""

# Scenario H: new behavior (stripped) with ❯ prefix
SCENARIO_H_STRIPPED_REAL = """User: ❯ I want episodes to contain the explicit user messages (literal copies) it should carry them all with what the agent said -- if the agent said a bunch of shit it should only show the last thing (which typically carries what the user is replying to)"""

if __name__ == "__main__":
    print(f"Model: {MODEL}")
    print("Testing current prompt against problem scenarios...\n")

    run("A: old behavior (tool_use in transcript)", SCENARIO_A_OLD)
    run("B: new behavior (tool_use stripped)", SCENARIO_B_STRIPPED)
    run("C: mid-session with text replies", SCENARIO_C_MID)
    run("D: nudge-to-keep (title already correct)", SCENARIO_D_NUDGE)
    run("E: vague user message", SCENARIO_E_VAGUE)
    run("F: generic tool calls (realistic failure case)", SCENARIO_F_GENERIC_TOOLS)
    run("G: agent opens wrong files first", SCENARIO_G_WRONG_FILE)
    run("H: stripped + real ❯ prefix", SCENARIO_H_STRIPPED_REAL)
