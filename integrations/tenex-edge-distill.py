#!/usr/bin/env python3
"""tenex-edge activity distiller (LLM, gated).

Reads a burst of recent tool activity on stdin and prints ONE short, intent-level
line describing what the agent is doing (e.g. "exploring the integrations layout"
rather than "Running: find . -name *.toml"). Wire it up via
`TENEX_EDGE_DISTILL_CMD=python3 /path/to/tenex-edge-distill.py`.

Fast + cheap by design: tiny model, low max_tokens, short timeout. On ANY error
it prints nothing and exits 0, so tenex-edge falls back to its heuristic.

Key resolution (first hit wins):
  $TENEX_EDGE_OPENROUTER_KEY
  ~/.proactive-context/config.json  -> openrouter_api_key
  ~/.local/share/opencode/auth.json -> openrouter.key
Model: $TENEX_EDGE_DISTILL_MODEL (default: a cheap fast model).
"""
import json
import os
import sys
import urllib.request

MODEL = os.environ.get("TENEX_EDGE_DISTILL_MODEL", "openai/gpt-4o-mini")
SYSTEM = (
    "You summarize what a coding agent is currently doing, in at most 8 words, "
    "present tense, describing intent not mechanics, no trailing punctuation. "
    "Examples: 'fixing the auth bug', 'exploring the integrations layout', "
    "'running the test suite'. Output only the phrase."
)


def api_key():
    k = os.environ.get("TENEX_EDGE_OPENROUTER_KEY", "").strip()
    if k:
        return k
    try:
        d = json.load(open(os.path.expanduser("~/.proactive-context/config.json")))
        if d.get("openrouter_api_key", "").strip():
            return d["openrouter_api_key"].strip()
    except Exception:
        pass
    try:
        d = json.load(open(os.path.expanduser("~/.local/share/opencode/auth.json")))
        oc = d.get("openrouter", {})
        if isinstance(oc, dict) and oc.get("key"):
            return oc["key"]
    except Exception:
        pass
    return None


def main():
    burst = sys.stdin.read().strip()
    if not burst:
        return
    key = api_key()
    if not key:
        return
    body = json.dumps({
        "model": MODEL,
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": burst},
        ],
        "max_tokens": 24,
        "temperature": 0.3,
    }).encode()
    req = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=body,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.load(resp)
        line = data["choices"][0]["message"]["content"].strip().splitlines()[0].strip()
        # tidy: drop surrounding quotes, cap length
        line = line.strip('"').strip("'")[:80]
        if line:
            print(line)
    except Exception:
        return  # silent → tenex-edge uses its heuristic


if __name__ == "__main__":
    main()
