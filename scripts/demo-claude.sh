#!/usr/bin/env bash
# Live demo driven by a REAL Claude Code session.
#
# A real `claude -p` run, with tenex-edge wired into its hooks, becomes a citizen
# on the fabric: it publishes presence and — once its turn runs past the distill
# threshold — its activity is distilled from the conversation transcript and
# published, all visible in a live `tenex-edge tail`. UserPromptSubmit marks the
# turn working (turn-start); Stop ends it (turn-end).
#
# NOTE: Claude Code merges this run's `--settings` with the user's GLOBAL
# ~/.claude/settings.json. On a machine that already has tenex-edge installed
# globally, those global hooks ALSO fire and their TENEX_EDGE_AGENT (e.g.
# "claude") overrides this demo's "claude-coder" export — so the published slug
# may be "claude". The assertions below accept either slug, so the demo stays
# green either way; the published agent is just named per the effective env.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/debug/tenex-edge"
HOOK="$ROOT/integrations/claude-code/te-hook.py"
NAK="${NAK:-$HOME/go/bin/nak}"; [ -x "$NAK" ] || NAK=nak

TMP="$(mktemp -d)"
PROJ="$TMP/myproject"; mkdir -p "$PROJ"
echo "The secret pass phrase is platypus-42." > "$PROJ/notes.txt"

export TENEX_EDGE_BIN="$BIN"
export TENEX_EDGE_AGENT="claude-coder"
export TENEX_EDGE_HOME="$TMP/edgehome"
export TENEX_CONFIG="$TMP/config.json"
export TENEX_EDGE_HEARTBEAT_MS=1000
export TENEX_EDGE_OBS_MS=700
export TENEX_EDGE_STATUS_TTL_S=30
# Distill 1s into the turn (default is 30s) so even this brief claude turn gets
# summarized before it ends. Activity distillation is LLM-only; point the seam at
# a deterministic stub that reads the real transcript (proving the path) instead
# of needing an API key. Production uses the `edge-distillation` rig role.
export TENEX_EDGE_TURN_FIRST_S=1
export TENEX_EDGE_DISTILL_CMD='in=$(cat); echo "$in" | grep -qi "notes.txt" && echo "Reading the project notes"'

PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')
RELAY="ws://localhost:$PORT"
cat >"$TENEX_CONFIG" <<EOF
{ "whitelistedPubkeys": [], "relays": ["$RELAY"], "backendName": "demo-box" }
EOF

SETTINGS="$TMP/settings.json"
sed "s#__HOOK__#$HOOK#g" "$ROOT/integrations/claude-code/settings.template.json" > "$SETTINGS"

cleanup() {
  pkill -f "$BIN __run-session" 2>/dev/null
  [ -n "${TAIL_PID:-}" ] && kill "$TAIL_PID" 2>/dev/null
  [ -n "${NAK_PID:-}" ] && kill "$NAK_PID" 2>/dev/null
  rm -rf "$TMP"
}
trap cleanup EXIT

echo "== starting relay $RELAY =="
"$NAK" serve --port "$PORT" --quiet >/dev/null 2>&1 &
NAK_PID=$!
for _ in $(seq 1 100); do
  python3 -c "import socket,sys;sys.exit(0 if socket.socket().connect_ex(('127.0.0.1',$PORT))==0 else 1)" && break
  sleep 0.05
done

echo "== starting tail (live fabric view) =="
TAIL_LOG="$TMP/tail.log"
"$BIN" tail --project myproject >"$TAIL_LOG" 2>/dev/null &
TAIL_PID=$!
sleep 0.5

echo "== running REAL claude -p session (hooks -> tenex-edge) =="
CLAUDE_OUT="$TMP/claude.out"
( cd "$PROJ" && claude -p \
    "Use the Read tool to read the file notes.txt in this directory, then reply with exactly the secret pass phrase it contains." \
    --settings "$SETTINGS" \
    --allowedTools Read \
    --dangerously-skip-permissions \
    >"$CLAUDE_OUT" 2>"$TMP/claude.err" )
echo "   claude replied: $(cat "$CLAUDE_OUT")"

sleep 1.5

echo
echo "== live tail captured (a REAL claude session on the fabric) =="
sed 's/^/   | /' "$TAIL_LOG"

echo
echo "== assertions =="
fail=0
strip() { sed $'s/\x1b\\[[0-9;]*m//g'; }
TAIL_PLAIN="$(strip < "$TAIL_LOG")"
check() { if echo "$2" | grep -q "$3"; then echo "  PASS: $1"; else echo "  FAIL: $1"; fail=1; fi; }
check "claude actually answered (read the file)" "$(cat "$CLAUDE_OUT")" "platypus-42"
# Slug match is intentionally loose: a machine with tenex-edge installed globally
# sets TENEX_EDGE_AGENT (e.g. "claude"), which wins over this demo's "claude-coder"
# export. We only care that a real claude-driven agent appears on the fabric with
# distilled activity — not what it's named — so accept "claude" or "claude-coder".
check "tail shows the claude agent's identity"   "$TAIL_PLAIN" "claude\(-coder\)\?@"
check "tail shows the claude agent live"         "$TAIL_PLAIN" "live .*claude\(-coder\)\?@myproject"
check "tail shows distilled activity from transcript" "$TAIL_PLAIN" "act .*claude\(-coder\)\?.*Reading the project notes"

echo
if [ "$fail" = 0 ]; then echo "ALL CHECKS PASSED"; else echo "SOME CHECKS FAILED"; fi
exit $fail
