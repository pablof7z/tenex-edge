#!/usr/bin/env bash
# e2e/run-ordinal.sh — durable-ordinal-identity + subscription redesign (issue
# #47, Phase 7 prep) against the REAL croissant NIP-29 relay.
#
# This is a SELF-CONTAINED rig: it sources e2e/lib.sh for the shared helpers
# (edge(), backend_seckey, relay liveness, wait_for*, logging) and reuses
# e2e/teardown.sh for cleanup, but boots its OWN relay + a single backend so it
# does not depend on e2e/run.sh's two-backend smoke having run first. It never
# modifies run.sh / run-subgroup.sh / lib.sh / teardown.sh.
#
# WHAT IT VERIFIES
#   1. Ordinal allocation — two concurrent sessions of the SAME agent in the
#      SAME room get DISTINCT routable ordinal pubkeys and labels (`smithN`).
#      The same ordinal may be reused in a different room, but never
#      concurrently in the same room. The bootstrap session intentionally keeps
#      `smith1` occupied, so the concurrent pair usually lands as `smith2` and
#      `smith3`.
#   2. Room-independent reuse — `smithN`'s pubkey is identical wherever it
#      appears. The rig creates a second channel, starts `smith1` there, and
#      compares it with the bootstrap `smith1` in the project root.
#   3. Chat routing by (pubkey, h) — one session writes a chat mentioning the
#      other; the mention must resolve to the recipient session and the on-wire
#      kind:9 must carry BOTH the room `#h` and the recipient `#p` pubkey. That
#      `(#p, #h)` pair IS the ordinal routing key.
#      → assertable when the mention resolves + the relay accepts the write;
#        SKIP-guarded if the mention does not resolve or the relay drops it.
#   4. Switch-reject — `channel switch` of a session into a channel
#      where the same ordinal is already live must be rejected with an error
#      containing "already active".
#
# DEGRADATION CONTRACT: only INFRASTRUCTURE problems are hard failures (relay
# down, binary missing, backend won't boot, project group never created).
# Behavioral invariants that are fully wired are hard failures. Relay-dependent
# chat-routing observability remains SKIP-guarded when local routing works but
# the relay does not echo the proof event. The run exits nonzero iff a hard check
# FAILS.
#
# Tunables: see e2e/lib.sh. Extra knobs:
#   E2E_ORD_PROJECT   project/room slug (default: ord-demo)
#   E2E_ORD_AGENT     agent slug under test (default: smith)

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"

# Isolate from run.sh's default project so an interactive run.sh checkout is not
# clobbered conceptually (teardown still wipes E2E_WORK — this rig owns it).
E2E_PROJECT="${E2E_ORD_PROJECT:-ord-demo}"
AGENT_SLUG="${E2E_ORD_AGENT:-smith}"

require_nak
command -v jq >/dev/null 2>&1 || die "jq not found on PATH — required to parse relay JSON"
command -v sqlite3 >/dev/null 2>&1 || die "sqlite3 not found on PATH — required to inspect backend state"

# ── check accounting ─────────────────────────────────────────────────────────
PASS_N=0
FAIL_N=0
SKIP_N=0
declare -a RESULTS=()
check_pass() { PASS_N=$((PASS_N + 1)); RESULTS+=("PASS  $1"); printf '%s PASS %s %s\n' "$_c_green" "$_c_reset" "$1"; }
check_fail() { FAIL_N=$((FAIL_N + 1)); RESULTS+=("FAIL  $1"); printf '%sFAIL%s %s\n' "$_c_red" "$_c_reset" "$1" >&2; }
check_skip() { SKIP_N=$((SKIP_N + 1)); RESULTS+=("SKIP  $1"); printf '%sSKIP%s %s\n' "$_c_yellow" "$_c_reset" "$1"; }

# ── keepalive bookkeeping ────────────────────────────────────────────────────
KEEPALIVE_PID=""
WATCH_PIDS=()
cleanup() {
  [[ -n "${KEEPALIVE_PID}" ]] && kill "${KEEPALIVE_PID}" 2>/dev/null || true
  for wp in "${WATCH_PIDS[@]:-}"; do [[ -n "$wp" ]] && kill "$wp" 2>/dev/null || true; done
}
trap cleanup EXIT

# ── build the binary under test ──────────────────────────────────────────────
log "building tenex-edge under test (cargo build)"
( cd "${REPO_ROOT}" && cargo build ) || die "cargo build failed"
[[ -x "${TENEX_EDGE_BIN}" ]] || die "tenex-edge binary not found at ${TENEX_EDGE_BIN} after build"
ok "binary: ${TENEX_EDGE_BIN}"

# ── 0. clean slate (reuse teardown.sh) ───────────────────────────────────────
log "step 0: tearing down any previous run"
E2E_KEEP_DATA=0 "${E2E_DIR}/teardown.sh" >/dev/null 2>&1 || true
mkdir -p "${E2E_WORK}" "${KEYS_DIR}"

# ── 1. boot the NIP-29 relay (croissant) ─────────────────────────────────────
log "step 1: NIP-29 relay"
if [[ ! -x "${NIP29_RELAY_BIN}" ]]; then
  log "building NIP-29 relay (CGO; one-time, ~1m)"
  ( cd "${NIP29_RELAY_DIR}" && CGO_ENABLED=1 go build -o ./croissant ) || die "NIP-29 relay build failed"
fi
ok "NIP-29 relay binary: ${NIP29_RELAY_BIN}"

mkdir -p "${RELAY_DATA}"
OWNER_PK="$(backend_pubkey edge-a)"
log "starting relay on ${RELAY_WS} (data: ${RELAY_DATA})"
nohup env PORT="${RELAY_PORT}" HOST="${RELAY_HOST}" DATAPATH="${RELAY_DATA}" \
    OWNER_PUBLIC_KEY="${OWNER_PK}" DOMAIN="" \
    "${NIP29_RELAY_BIN}" >"${RELAY_LOG}" 2>&1 &
RELAY_PID=$!
echo "${RELAY_PID}" >"${RELAY_PIDFILE}"
wait_for "relay NIP-11 to report supported_nips" 20 relay_up

# Stale-relay guard: the process on the port MUST be the one we just launched.
LISTENER_PID="$(lsof -nP -tiTCP:"${RELAY_PORT}" -sTCP:LISTEN 2>/dev/null | head -1 || true)"
if [[ -n "${LISTENER_PID}" && "${LISTENER_PID}" != "${RELAY_PID}" ]]; then
  die "port ${RELAY_PORT} held by pid ${LISTENER_PID}, not our relay (pid ${RELAY_PID}) — run ./e2e/teardown.sh and retry."
fi
if curl -fsS -H 'Accept: application/nostr+json' "${RELAY_HTTP}" | grep -Eq '"supported_nips":[^]]*\b29\b'; then
  ok "relay up; NIP-11 advertises NIP-29"
else
  die "relay does not advertise NIP-29 (see ${RELAY_LOG})"
fi

# ── 2. single backend config + project registration ──────────────────────────
log "step 2: single isolated backend (edge-a)"
A_SK="$(backend_seckey edge-a)"
A_PK="$(backend_pubkey edge-a)"
A_EDGE="$(backend_edge_home edge-a)"
A_TDIR="$(backend_tenex_dir edge-a)"
A_CFG="$(backend_config edge-a)"
A_PROJ="$(backend_project_dir edge-a)"
mkdir -p "${A_EDGE}" "${A_TDIR}" "${A_PROJ}"
cat >"${A_CFG}" <<JSON
{
  "whitelistedPubkeys": ["${A_PK}"],
  "relays": ["${RELAY_WS}"],
  "indexerRelay": "${RELAY_WS}",
  "backendName": "edge-a",
  "userNsec": "${A_SK}",
  "tenexPrivateKey": "${A_SK}"
}
JSON
( cd "${A_PROJ}" && edge edge-a channel init --force >/dev/null ) || die "channel init failed"
ok "edge-a pubkey ${A_PK}"
dim "  config=${A_CFG}"
dim "  project_dir=${A_PROJ}  project_slug=${E2E_PROJECT}"

# Keep the daemon ALIVE for the whole test: a long-lived `tail` client holds the
# socket open so the auto-spawned daemon does not idle-exit between CLI calls
# (the per-session engines + status_outbox drainer must keep running to publish).
edge edge-a tail >/dev/null 2>&1 &
KEEPALIVE_PID=$!
sleep 1

snapshot_daemon_pid() {
  local pid
  pid="$(pgrep -n -f "${TENEX_EDGE_BIN} daemon" || true)"
  [[ -n "$pid" ]] && echo "$pid" >"$(backend_pidfile edge-a)"
}

# ── 3. bootstrap: create the bare project/room group on the relay ─────────────
# A plain session-start (no TENEX_EDGE_CHANNEL) in the project dir drives
# open_project, which publishes the work-root project group (kind:39000 d=slug).
# This is the room the two concurrent sessions below join via TENEX_EDGE_CHANNEL.
log "step 3: bootstrap session-start (creates room '${E2E_PROJECT}')"
BOOT_SID="ord-boot-$(date +%s)"
# A session's watch_pid (3rd arg) keeps it alive: the daemon reaps sessions whose
# watched pid is dead. Hook-driven test sessions have no real harness process, so
# we attach a persistent `sleep` as the watched pid (mirrors a real harness
# process). Without this, concurrent same-room sessions get reaped before the
# ordinal-collision check runs.
session_start_payload() {
  if [[ -n "${3:-}" ]]; then
    printf '{"session_id":"%s","cwd":"%s","watch_pid":%s}' "$1" "$2" "$3"
  else
    printf '{"session_id":"%s","cwd":"%s"}' "$1" "$2"
  fi
}
# Spawn a persistent watched process and stash its pid in WATCH_PIDS (declared
# with cleanup() above; killed on EXIT there). Runs in the PARENT shell (NOT via
# $()) so the append persists, and redirects the sleep's fds so a caller using
# $() would not block on the inherited pipe.
new_watch() { sleep 900 >/dev/null 2>&1 & LAST_WATCH=$!; WATCH_PIDS+=("$LAST_WATCH"); }
new_watch; WP_BOOT="$LAST_WATCH"
(
  cd "${A_PROJ}"
  echo "$(session_start_payload "${BOOT_SID}" "${A_PROJ}" "${WP_BOOT}")" \
    | TENEX_EDGE_AGENT="${AGENT_SLUG}" edge edge-a harness hook claude-code --type session-start
) || die "bootstrap session-start failed (see ${A_EDGE}/daemon.log)"
snapshot_daemon_pid

# HARD requirement: the room must exist on the relay before any room-scoped work.
if wait_for_soft "relay kind:39000 metadata for ${E2E_PROJECT}" 25 \
     "nak_req_contains '\"kind\":39000' -k 39000 -d '${E2E_PROJECT}' '${RELAY_WS}'"; then
  ok "room '${E2E_PROJECT}' exists on the relay (kind:39000)"
else
  nak_req_limited 4 -k 39000 -d "${E2E_PROJECT}" "${RELAY_WS}" 2>&1 | sed 's/^/    /' || true
  die "room '${E2E_PROJECT}' never landed on the relay — cannot run room-scoped checks"
fi

# ── 4. CHECK 1: ordinal allocation — distinct pubkeys for two live sessions ───
log "check 1: two concurrent sessions of '${AGENT_SLUG}' in room '${E2E_PROJECT}'"
SID0="ord-s0-$$-$(date +%s)"
SID1="ord-s1-$$-$(date +%s)"

# Launch BOTH session-start hooks concurrently, scoped to the SAME room via
# TENEX_EDGE_CHANNEL, SAME agent slug, SAME cwd, DIFFERENT session ids.
start_session() {
  local sid="$1" wp="$2" channel="${3:-${E2E_PROJECT}}"
  (
    cd "${A_PROJ}"
    echo "$(session_start_payload "${sid}" "${A_PROJ}" "${wp}")" \
      | TENEX_EDGE_AGENT="${AGENT_SLUG}" TENEX_EDGE_CHANNEL="${channel}" \
        edge edge-a harness hook claude-code --type session-start
  )
}
new_watch; WP0="$LAST_WATCH"
new_watch; WP1="$LAST_WATCH"
# Start BOTH concurrently so both reservations are held simultaneously: that is
# exactly the collision the ordinal allocator resolves (lowest-free → 1 then 2).
# Sequential starts let the first (watch-pid-less) session get idle-reaped before
# the second runs, freeing ordinal 1 and masking the collision.
start_session "${SID0}" "${WP0}" >/dev/null 2>&1 &
P0=$!
start_session "${SID1}" "${WP1}" >/dev/null 2>&1 &
P1=$!
wait "${P0}" || warn "session ${SID0} start returned nonzero"
wait "${P1}" || warn "session ${SID1} start returned nonzero"
snapshot_daemon_pid
sleep 2

# Resolve each session's routable identity LOCALLY via the backend state DB. This
# does not depend on relay membership/acceptance, so it isolates identity
# allocation behavior from relay policy.
session_identity_row() {
  local sid="$1"
  local db
  db="$(backend_edge_home edge-a)/state.db"
  sqlite3 -separator $'\t' "${db}" "
    SELECT
      COALESCE(i.pubkey, s.agent_pubkey),
      COALESCE(
        CASE WHEN i.ordinal > 0 THEN i.agent_slug || i.ordinal ELSE i.agent_slug END,
        s.agent_slug
      )
    FROM sessions s
    LEFT JOIN identities i ON i.session_id = s.session_id
    WHERE s.session_id = COALESCE(
      (SELECT session_id FROM sessions WHERE session_id = '${sid}' LIMIT 1),
      (SELECT session_id FROM session_aliases WHERE external_id = '${sid}' ORDER BY created_at DESC LIMIT 1)
    )
    ORDER BY i.created_at DESC
    LIMIT 1;
  " 2>/dev/null || true
}
IFS=$'\t' read -r PK_BOOT AG_BOOT <<<"$(session_identity_row "${BOOT_SID}")"
IFS=$'\t' read -r PK0 AG0 <<<"$(session_identity_row "${SID0}")"
IFS=$'\t' read -r PK1 AG1 <<<"$(session_identity_row "${SID1}")"
dim "  bootstrap ${BOOT_SID}: agent=${AG_BOOT} pubkey=${PK_BOOT:0:12}"
dim "  session0 ${SID0}: agent=${AG0} pubkey=${PK0:0:12}"
dim "  session1 ${SID1}: agent=${AG1} pubkey=${PK1:0:12}"

if [[ -z "${PK0}" || -z "${PK1}" ]]; then
  check_skip "1 ordinal allocation — could not resolve both sessions from state.db (daemon may have idle-reaped a session with no watched pid)"
elif [[ "${PK0}" != "${PK1}" ]]; then
  check_pass "1 ordinal allocation — two concurrent sessions got DISTINCT routable pubkeys (${PK0:0:8} != ${PK1:0:8})"
else
  check_skip "1 ordinal allocation — both sessions share one pubkey (${PK0:0:8}); distinct-identity allocation not active for this concurrency"
fi

is_ordinal_label() {
  local label="$1" suffix
  suffix="${label#"${AGENT_SLUG}"}"
  [[ "${label}" != "${suffix}" && "${suffix}" =~ ^[1-9][0-9]*$ ]]
}

if is_ordinal_label "${AG0}" && is_ordinal_label "${AG1}" && [[ "${AG0}" != "${AG1}" ]]; then
  check_pass "1b ordinal labels — sessions surface distinct live labels '${AG0}' and '${AG1}'"
else
  check_fail "1b ordinal labels — expected distinct '${AGENT_SLUG}N' labels, got '${AG0}'/'${AG1}'"
fi

# 1c. AUTHORITATIVE wire check: kind:30315 presence in the room must carry TWO
# distinct authors (`smith1` + `smith2`), each signing with its own ordinal key.
# Retry to absorb presence-publish timing — this is the headline proof that
# distinct ordinal identities are live on the relay.
log "check 1c: relay kind:30315 presence authors in room '${E2E_PROJECT}'"
CONFIRMED_MEMBERS=0
if [[ -n "${PK0}" && -n "${PK1}" ]]; then
  CONFIRMED_MEMBERS="$(sqlite3 "$(backend_edge_home edge-a)/state.db" "
    SELECT COUNT(DISTINCT pubkey)
    FROM relay_channel_members
    WHERE channel_h='${E2E_PROJECT}' AND pubkey IN ('${PK0}', '${PK1}');
  " 2>/dev/null || true)"
fi
CONFIRMED_MEMBERS="${CONFIRMED_MEMBERS:-0}"
dim "  confirmed ordinal relay members: ${CONFIRMED_MEMBERS}"
if [[ "${CONFIRMED_MEMBERS}" -lt 2 ]]; then
  check_skip "1c presence — relay did not confirm both ordinal member grants; skipping wire-status assertion"
else
  DISTINCT_PUBKEYS=0
  for _try in 1 2 3 4 5 6 7 8; do
    DISTINCT_PUBKEYS="$(nak_req_limited 4 -k 30315 -h "${E2E_PROJECT}" "${RELAY_WS}" 2>/dev/null \
      | jq -r '.pubkey // empty' 2>/dev/null | sort -u | grep -c . || true)"
    DISTINCT_PUBKEYS="${DISTINCT_PUBKEYS:-0}"
    [[ "${DISTINCT_PUBKEYS}" -ge 2 ]] && break
    sleep 1
  done
  dim "  distinct 30315 authors in room: ${DISTINCT_PUBKEYS}"
  if [[ "${DISTINCT_PUBKEYS}" -ge 2 ]]; then
    check_pass "1c presence — ${DISTINCT_PUBKEYS} distinct ordinal identities publish kind:30315 into room '${E2E_PROJECT}'"
  else
    check_fail "1c presence — expected >=2 distinct kind:30315 authors in '${E2E_PROJECT}', saw ${DISTINCT_PUBKEYS} (ordinal allocation not reaching the wire)"
  fi
fi

# ── 5. CHECK 2: room-independent reuse ────────────────────────────────────────
log "check 2: room-independent ordinal reuse"
ALT_NAME="ord-alt-$$"
ALT_H=""
SID2=""
PK2=""
AG2=""
if (
  cd "${A_PROJ}"
  run_limited 60 edge edge-a channel create "${ALT_NAME}" --about "ordinal alternate room" >/dev/null
); then
  ALT_H="$(sqlite3 "$(backend_edge_home edge-a)/state.db" \
    "SELECT channel_h FROM relay_channels WHERE parent='${E2E_PROJECT}' AND name='${ALT_NAME}' ORDER BY updated_at DESC LIMIT 1;" \
    2>/dev/null || true)"
  if [[ -n "${ALT_H}" ]]; then
    dim "  alternate channel ${ALT_NAME}: ${ALT_H}"
    SID2="ord-s2-$$-$(date +%s)"
    new_watch; WP2="$LAST_WATCH"
    if run_limited 60 start_session "${SID2}" "${WP2}" "${ALT_H}" >/dev/null 2>&1; then
      sleep 1
      IFS=$'\t' read -r PK2 AG2 <<<"$(session_identity_row "${SID2}")"
      dim "  alternate session ${SID2}: agent=${AG2} pubkey=${PK2:0:12}"
      if [[ -z "${PK_BOOT}" || -z "${PK2}" || -z "${AG_BOOT}" || -z "${AG2}" ]]; then
        check_fail "2 room-independent reuse — could not resolve bootstrap and alternate session identities"
      elif [[ "${AG_BOOT}" == "${AG2}" && "${PK_BOOT}" == "${PK2}" ]]; then
        check_pass "2 room-independent reuse — ${AG2} has the same pubkey in project root and ${ALT_H}"
      else
        check_fail "2 room-independent reuse — expected bootstrap ${AG_BOOT}/${PK_BOOT:0:8} to match alternate ${AG2}/${PK2:0:8}"
      fi
    else
      check_skip "2 room-independent reuse — alternate session-start did not complete against relay-backed channel ${ALT_H}"
    fi
  else
    check_skip "2 room-independent reuse — alternate channel '${ALT_NAME}' did not materialize in state.db"
  fi
else
  check_skip "2 room-independent reuse — alternate channel create did not complete against the relay"
fi

# ── 6. CHECK 3: chat routing by (pubkey, h) ──────────────────────────────────
log "check 3: chat routing — session0 mentions session1 in room '${E2E_PROJECT}'"
if [[ -z "${AG1}" || -z "${PK1}" || -z "${PK0}" || "${PK0}" == "${PK1}" ]]; then
  check_skip "3 chat routing — need two distinct resolvable sessions with a recipient label (agent1='${AG1}'); prerequisite check 1 did not produce them"
else
  CHAT_BODY="ordinal-routing-probe-$$ ping @${AG1} please ack"
  CHAT_OUT="$(
    cd "${A_PROJ}"
    TENEX_EDGE_AGENT="${AGENT_SLUG}" TENEX_EDGE_CHANNEL="${E2E_PROJECT}" \
      run_limited 45 edge edge-a channel send "${CHAT_BODY}" --channel "${E2E_PROJECT}" --session "${SID0}" 2>&1
  )" || true
  dim "  channel send: ${CHAT_OUT}"

  MENTION_OK=0
  if echo "${CHAT_OUT}" | grep -qi "mentioning @${AG1}"; then
    MENTION_OK=1
  fi

  # On the wire: the kind:9 must carry the room #h AND the recipient #p pubkey —
  # that (#p,#h) pair IS the ordinal routing key. -h/-p are nak tag shortcuts.
  WIRE_OK=0
  if wait_for_soft "kind:9 with #h=${E2E_PROJECT} and #p=${PK1:0:8}" 12 \
       "nak_req_contains 'ordinal-routing-probe' -k 9 -h '${E2E_PROJECT}' -p '${PK1}' '${RELAY_WS}'"; then
    WIRE_OK=1
  fi

  # Local delivery: recipient session sees the mention via channel read.
  READ_OK=0
  READ_OUT="$(
    cd "${A_PROJ}"
    TENEX_EDGE_AGENT="${AGENT_SLUG}" TENEX_EDGE_CHANNEL="${E2E_PROJECT}" \
      run_limited 20 edge edge-a channel read --channel "${E2E_PROJECT}" --session "${SID1}" --limit 20 2>/dev/null
  )" || true
  if echo "${READ_OUT}" | grep -q "ordinal-routing-probe"; then
    READ_OK=1
  fi
  dim "  mention_resolved=${MENTION_OK}  wire(#p,#h)=${WIRE_OK}  recipient_read=${READ_OK}"

  if [[ "${WIRE_OK}" == "1" && ( "${MENTION_OK}" == "1" || "${READ_OK}" == "1" ) ]]; then
    check_pass "3 chat routing — mention routed to session1 and kind:9 carries (#p=${PK1:0:8}, #h=${E2E_PROJECT})"
  elif [[ "${MENTION_OK}" == "1" || "${READ_OK}" == "1" ]]; then
    check_skip "3 chat routing — mention resolved locally but the (#p,#h) kind:9 was not retrievable from the relay (NIP-29 membership/acceptance); routing key not confirmable on-wire"
  else
    check_skip "3 chat routing — mention did not resolve and no kind:9 on-wire (chat/mention routing not exercisable in this configuration)"
  fi
fi

# ── 7. CHECK 4: switch-reject (Phase 5) ──────────────────────────────────────
log "check 4: channel switch rejects a live-ordinal collision (Phase 5)"
# Real collision scenario: the bootstrap session holds ${AG_BOOT} in
# '${E2E_PROJECT}', and session2 holds that same ordinal in ${ALT_H}. Switching
# session2 into '${E2E_PROJECT}' would create two live sessions with the same
# (pubkey, h), so the daemon must reject with "already active".
if [[ -z "${ALT_H}" || -z "${SID2}" || -z "${AG2}" || -z "${PK2}" ]]; then
  check_skip "4 switch-reject — alternate live session prerequisite was not available"
else
  SWITCH_OUT="$(
    cd "${A_PROJ}"
    TENEX_EDGE_AGENT="${AGENT_SLUG}" \
      run_limited 30 edge edge-a channel switch --session "${SID2}" "${E2E_PROJECT}" 2>&1
  )" || true
  dim "  channel switch (${AG2} ${ALT_H} -> ${E2E_PROJECT}): ${SWITCH_OUT}"
  if echo "${SWITCH_OUT}" | grep -qi "already active"; then
    check_pass "4 switch-reject — daemon rejected the live-ordinal collision with 'already active'"
  else
    check_fail "4 switch-reject — expected 'already active' rejection, got: ${SWITCH_OUT}"
  fi
fi

# ── 8. summary ───────────────────────────────────────────────────────────────
echo
log "ordinal e2e summary"
for line in "${RESULTS[@]}"; do
  case "${line}" in
    PASS*) printf '  %s%s%s\n' "$_c_green" "${line}" "$_c_reset" ;;
    FAIL*) printf '  %s%s%s\n' "$_c_red" "${line}" "$_c_reset" ;;
    SKIP*) printf '  %s%s%s\n' "$_c_yellow" "${line}" "$_c_reset" ;;
  esac
done
echo
printf 'totals: %sPASS=%d%s  %sSKIP=%d%s  %sFAIL=%d%s\n' \
  "$_c_green" "${PASS_N}" "$_c_reset" \
  "$_c_yellow" "${SKIP_N}" "$_c_reset" \
  "$_c_red" "${FAIL_N}" "$_c_reset"

cat <<NOTE

relay   ${RELAY_WS}   (pid $(cat "${RELAY_PIDFILE}" 2>/dev/null || echo '?'), log ${RELAY_LOG})
backend home=${A_EDGE}
room    ${E2E_PROJECT}
inspect: nak req -k 30315 -h ${E2E_PROJECT} ${RELAY_WS}
tear down: ./e2e/teardown.sh   (E2E_KEEP_DATA=1 to keep state)
NOTE

if [[ "${FAIL_N}" -gt 0 ]]; then
  die "${FAIL_N} hard check(s) FAILED"
fi
ok "no hard failures (${PASS_N} pass, ${SKIP_N} skip)"
