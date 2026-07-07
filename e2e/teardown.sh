#!/usr/bin/env bash
# e2e/teardown.sh — stop the relay + both backend daemons and (by default) wipe
# all ephemeral state. Idempotent: safe to run when nothing is up.
#
#   ./e2e/teardown.sh           # kill everything + remove $E2E_WORK
#   E2E_KEEP_DATA=1 ./e2e/teardown.sh   # kill everything, keep data + keys

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

# Kill a process recorded in a pidfile (if alive), then remove the pidfile.
kill_pidfile() {
  local f="$1" label="$2"
  [[ -f "$f" ]] || return 0
  local pid; pid="$(cat "$f" 2>/dev/null || true)"
  if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
    log "stopping ${label} (pid ${pid})"
    kill "$pid" 2>/dev/null || true
    # give it a moment, then hard-kill
    for _ in 1 2 3 4 5 6 7 8 9 10; do kill -0 "$pid" 2>/dev/null || break; sleep 0.2; done
    kill -9 "$pid" 2>/dev/null || true
  fi
  rm -f "$f"
}

# Stop each backend's daemon. We recorded its pid in daemon.pid at boot, but the
# daemon detaches into its own process group, so also sweep by the unique socket
# path it was launched with (belt and suspenders, and catches stale daemons from
# a previous run whose pidfile we lost).
stop_backend() {
  local name="$1"
  kill_pidfile "$(backend_pidfile "$name")" "backend ${name} daemon"
  local sock; sock="$(backend_edge_home "$name")/daemon.sock"
  # pkill matches the daemon by the socket dir it serves (each home is unique).
  pkill -f "${TENEX_EDGE_BIN} __daemon" 2>/dev/null && true
  rm -f "$sock"
}

stop_pty_supervisors() {
  local pids pid
  pids="$(ps -axo pid=,command= | awk -v bin="${TENEX_EDGE_BIN}" -v work="${E2E_WORK}" '
    index($0, bin " __pty-supervisor") && index($0, work) { print $1 }
  ')"
  [[ -n "${pids}" ]] || return 0
  for pid in ${pids}; do
    if kill -0 "${pid}" 2>/dev/null; then
      log "stopping e2e PTY supervisor (pid ${pid})"
      kill "${pid}" 2>/dev/null || true
    fi
  done
  sleep 0.5
  for pid in ${pids}; do
    kill -0 "${pid}" 2>/dev/null && kill -9 "${pid}" 2>/dev/null || true
  done
}

log "tearing down tenex-edge e2e rig"

stop_pty_supervisors
stop_backend edge-a
stop_backend edge-b
kill_pidfile "${RELAY_PIDFILE}" "NIP-29 relay"

# Also reclaim the relay PORT: an orphan relay from a manual launch or a crashed
# prior run (no pidfile we own) would otherwise keep $RELAY_PORT bound, and the
# next run.sh would silently talk to that STALE relay. Kill whatever holds it.
port_pids="$(lsof -nP -tiTCP:"${RELAY_PORT}" -sTCP:LISTEN 2>/dev/null || true)"
if [[ -n "${port_pids}" ]]; then
  for pid in ${port_pids}; do
    log "reclaiming port ${RELAY_PORT} from pid ${pid}"
    kill "${pid}" 2>/dev/null || true
  done
  sleep 0.5
  port_pids="$(lsof -nP -tiTCP:"${RELAY_PORT}" -sTCP:LISTEN 2>/dev/null || true)"
  for pid in ${port_pids}; do kill -9 "${pid}" 2>/dev/null || true; done
fi

# Final sweep: any __daemon launched from THIS binary that survived.
if pgrep -f "${TENEX_EDGE_BIN} __daemon" >/dev/null 2>&1; then
  warn "sweeping stray __daemon processes"
  pkill -9 -f "${TENEX_EDGE_BIN} __daemon" 2>/dev/null || true
fi

stop_pty_supervisors

if [[ "${E2E_KEEP_DATA:-0}" == "1" ]]; then
  ok "processes stopped; data kept at ${E2E_WORK}"
else
  rm -rf "${E2E_WORK}"
  ok "processes stopped; ${E2E_WORK} removed"
fi
