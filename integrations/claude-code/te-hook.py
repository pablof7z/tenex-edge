#!/usr/bin/env python3
"""tenex-edge ↔ Claude Code hook dispatcher.

Claude Code delivers hook input as JSON on stdin (session_id, cwd, tool_name,
tool_input, …). This thin adapter translates each hook event into a host-neutral
`tenex-edge` CLI call. tenex-edge knows nothing about Claude Code; this file is
the straw, not the milkshake.

Wire it up in settings.json (see settings.template.json). Env:
  TENEX_EDGE_BIN    path to the tenex-edge binary (default: "tenex-edge")
  TENEX_EDGE_AGENT  slug this host's agent goes by (default: "claude")
"""
import sys
import os
import json
import subprocess


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    binary = os.environ.get("TENEX_EDGE_BIN", "tenex-edge")
    try:
        data = json.load(sys.stdin)
    except Exception:
        data = {}
    sid = data.get("session_id", "")
    cwd = data.get("cwd", "") or os.getcwd()
    transcript = data.get("transcript_path", "") or ""

    def run(args, capture=False):
        try:
            return subprocess.run(
                [binary] + args,
                stdout=(subprocess.PIPE if capture else subprocess.DEVNULL),
                stderr=subprocess.DEVNULL,
                text=True,
            )
        except Exception:
            return None

    if mode == "session-start":
        agent = os.environ.get("TENEX_EDGE_AGENT", "claude")
        run(["session-start", "--agent", agent, "--session-id", sid, "--cwd", cwd])

    elif mode == "session-end":
        if sid:
            run(["session-end", "--session", sid])

    elif mode == "stop":
        # The agent finished its turn → go idle (engine clears status next poll).
        if sid:
            run(["turn-end", "--session", sid])

    elif mode == "user-prompt-submit":
        if not sid:
            return
        args = ["turn-start", "--session", sid]
        if transcript:
            args += ["--transcript", transcript]
        result = run(args, capture=True)
        if result and result.returncode == 0:
            out = (result.stdout or "").strip()
            if out:
                print(out)


if __name__ == "__main__":
    main()
