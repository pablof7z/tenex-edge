#!/usr/bin/env bash
# Opt-in real CLI probe for issue #278.
#
# This intentionally does not run in CI. It calls installed Claude/Codex CLIs
# and may spend account quota. Run only when explicitly requested:
#
#   MOSAICO_REAL_HEADLESS_E2E=1 scripts/e2e-headless-resume.sh
set -euo pipefail

if [ "${MOSAICO_REAL_HEADLESS_E2E:-}" != "1" ]; then
  echo "SKIP: set MOSAICO_REAL_HEADLESS_E2E=1 to run real Claude/Codex calls"
  exit 0
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

native_id() {
  python3 - "$1" <<'PY'
import json, sys

keys = ("session_id", "sessionId", "conversation_id", "conversationId", "thread_id", "threadId")

def find(value):
    if isinstance(value, list):
        for item in value:
            found = find(item)
            if found:
                return found
    if isinstance(value, dict):
        for key in keys:
            item = value.get(key)
            if isinstance(item, str) and item:
                return item
        session = value.get("session")
        if isinstance(session, dict):
            item = session.get("id")
            if isinstance(item, str) and item:
                return item
        for item in value.values():
            found = find(item)
            if found:
                return found
    return None

with open(sys.argv[1], encoding="utf-8") as fh:
    for line in fh:
        try:
            value = json.loads(line)
        except Exception:
            continue
        found = find(value)
        if found:
            print(found)
            sys.exit(0)
sys.exit(1)
PY
}

uuid_v4() {
  python3 - <<'PY'
import uuid
print(uuid.uuid4())
PY
}

run_claude() {
  need claude
  local session_id
  session_id="$(uuid_v4)"
  local fresh="$TMP/claude-fresh.jsonl"
  local resumed="$TMP/claude-resume.jsonl"

  (cd "$TMP" && claude -p \
    --session-id "$session_id" \
    --output-format json \
    --tools "" \
    --max-budget-usd "${CLAUDE_MAX_BUDGET_USD:-0.05}" \
    "Reply exactly: mosaico-claude-fresh" </dev/null >"$fresh")
  local fresh_id
  fresh_id="$(native_id "$fresh")"
  test "$fresh_id" = "$session_id"

  (cd "$TMP" && claude -p \
    --resume "$session_id" \
    --output-format json \
    --tools "" \
    --max-budget-usd "${CLAUDE_MAX_BUDGET_USD:-0.05}" \
    "Reply exactly: mosaico-claude-resume" </dev/null >"$resumed")
  local resumed_id
  resumed_id="$(native_id "$resumed")"
  test "$resumed_id" = "$session_id"
  echo "PASS: claude headless native resume id round trip ($session_id)"
}

run_codex() {
  need codex
  local fresh="$TMP/codex-fresh.jsonl"
  local resumed="$TMP/codex-resume.jsonl"

  (cd "$TMP" && codex exec \
    --json \
    --skip-git-repo-check \
    --sandbox read-only \
    "Reply exactly: mosaico-codex-fresh" </dev/null >"$fresh")
  local session_id
  session_id="$(native_id "$fresh")"

  (cd "$TMP" && codex exec \
    --json \
    --skip-git-repo-check \
    --sandbox read-only \
    resume "$session_id" \
    "Reply exactly: mosaico-codex-resume" </dev/null >"$resumed")
  local resumed_id
  resumed_id="$(native_id "$resumed")"
  test "$resumed_id" = "$session_id"
  echo "PASS: codex headless native resume id round trip ($session_id)"
}

echo "mosaico: $ROOT"
run_claude
run_codex
echo "ALL REAL HEADLESS CHECKS PASSED"
