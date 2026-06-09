#!/usr/bin/env python3
"""tenex-edge <-> Codex hook dispatcher.

Codex sends hook input as JSON on stdin. This adapter translates Codex hook
events into host-neutral `tenex-edge` CLI calls and prints turn context for
UserPromptSubmit. It must fail open: if tenex-edge is unavailable, Codex should
continue normally.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    binary = os.environ.get("TENEX_EDGE_BIN", "tenex-edge")
    try:
        data = json.load(sys.stdin)
    except Exception:
        data = {}

    sid = session_id(data)
    cwd = data.get("cwd", "") or os.getcwd()
    transcript = data.get("transcript_path", "") or ""

    if mode == "session-start":
        if not sid:
            log("session-start skipped: no session id in payload")
            return
        agent = os.environ.get("TENEX_EDGE_AGENT", "codex")
        args = ["session-start", "--agent", agent, "--session-id", sid, "--cwd", cwd]
        pid = find_codex_ancestor_pid()
        if pid:
            args += ["--watch-pid", str(pid)]
        result = run(binary, args)
        if result and result.returncode == 0:
            log(f"session-start ok: session={sid} cwd={cwd}")
            msg = (
                "[tenex-edge] Run `tenex-edge wait-for-mention` with run_in_background=true "
                "now so you are woken when a mention arrives. Re-run it each time one is received."
            )
            print(json.dumps({"systemMessage": msg}))
        else:
            code = result.returncode if result else "exec-error"
            log(f"session-start failed: code={code} session={sid} cwd={cwd}")

    elif mode == "user-prompt-submit":
        if not sid:
            log("user-prompt-submit skipped: no session id in payload")
            return
        args = ["turn-start", "--session", sid, "--json"]
        if transcript:
            args += ["--transcript", transcript]
        output = run_text(binary, args)
        if output:
            print(output)

    elif mode == "post-tool-use":
        if not sid:
            log("post-tool-use skipped: no session id in payload")
            return
        output = run_text(binary, ["turn-check", "--session", sid, "--json"])
        if output:
            print(output)

    elif mode == "stop":
        # Turn finished: mark the session idle. The engine decides whether the
        # turn ran long enough to have distilled anything.
        if not sid:
            log("stop skipped: no session id in payload")
            return
        run(binary, ["turn-end", "--session", sid])


def session_id(data):
    for key in (
        "session_id",
        "sessionId",
        "conversation_id",
        "conversationId",
        "thread_id",
        "threadId",
    ):
        value = data.get(key)
        if value:
            return str(value)
    return ""


def run(binary, args):
    try:
        return subprocess.run(
            [binary] + args,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            text=True,
            cwd=None,
        )
    except Exception:
        return None


def log(message):
    try:
        path = Path(os.environ.get("TENEX_EDGE_HOOK_LOG", "~/.tenex/edge/codex-hook.log")).expanduser()
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8") as f:
            f.write(message + "\n")
    except Exception:
        pass


def run_text(binary, args):
    try:
        result = subprocess.run(
            [binary] + args,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except Exception:
        return ""
    if result.returncode != 0:
        return ""
    return strip_ansi((result.stdout or "").strip())


def find_codex_ancestor_pid():
    """Return the nearest ancestor process whose command looks like Codex."""
    pid = os.getpid()
    seen = set()
    for _ in range(12):
        ppid = parent_pid(pid)
        if not ppid or ppid <= 1 or ppid in seen:
            return None
        seen.add(ppid)
        command = process_command(ppid)
        if command and "codex" in os.path.basename(command).lower():
            return ppid
        pid = ppid
    return None


def parent_pid(pid):
    try:
        out = subprocess.check_output(["ps", "-o", "ppid=", "-p", str(pid)], text=True)
        return int(out.strip())
    except Exception:
        return None


def process_command(pid):
    try:
        return subprocess.check_output(["ps", "-o", "comm=", "-p", str(pid)], text=True).strip()
    except Exception:
        return ""


def strip_ansi(s):
    return re.sub(r"\x1b\[[0-9;]*m", "", s)


if __name__ == "__main__":
    main()
