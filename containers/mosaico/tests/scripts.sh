#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

assert_mounts_only() {
  local agent="$1" expected="$2" mounts
  AGENT="${agent}"
  build_host_auth_mounts
  mounts="$(printf '%s\n' "${HOST_AUTH_MOUNTS[@]}")"
  grep -Fq "${expected}" <<<"${mounts}" || fail "${agent} auth mount missing"
  if grep -Fq '/host-auth/mosaico' <<<"${mounts}"; then
    fail "${agent} unexpectedly mounts host Mosaico state"
  fi
  case "${agent}" in
    claude)
      if grep -Eq '/host-auth/(codex|opencode)' <<<"${mounts}"; then
        fail 'Claude profile mounts an unrelated provider'
      fi
      ;;
    codex)
      if grep -Eq '/host-auth/(claude|opencode)' <<<"${mounts}"; then
        fail 'Codex profile mounts an unrelated provider'
      fi
      ;;
    opencode)
      if grep -Eq '/host-auth/(claude|codex)' <<<"${mounts}"; then
        fail 'OpenCode profile mounts an unrelated provider'
      fi
      ;;
  esac
}

HOST_HOME="${TMP}/host"
# Consumed by functions sourced below.
# shellcheck disable=SC2034
HOST_AUTH=1
mkdir -p \
  "${HOST_HOME}/.codex/agents" \
  "${HOST_HOME}/.claude/agents" \
  "${HOST_HOME}/.local/share/opencode" \
  "${HOST_HOME}/.config/opencode/agents"
printf '{}\n' >"${HOST_HOME}/.codex/auth.json"
printf '\n' >"${HOST_HOME}/.codex/config.toml"
printf '{}\n' >"${HOST_HOME}/.claude.json"
printf '{"claudeAiOauth":{"accessToken":"test","refreshToken":"test"}}\n' \
  >"${HOST_HOME}/.claude/.credentials.json"
printf '{}\n' >"${HOST_HOME}/.claude/settings.json"
printf '{}\n' >"${HOST_HOME}/.local/share/opencode/auth.json"
printf '{}\n' >"${HOST_HOME}/.local/share/opencode/account.json"
printf '{}\n' >"${HOST_HOME}/.config/opencode/opencode.jsonc"

# shellcheck source=/dev/null
source "${ROOT}/containers/mosaico/host-auth.bash"
assert_mounts_only claude '/host-auth/claude'
assert_mounts_only codex '/host-auth/codex'
assert_mounts_only opencode '/host-auth/opencode-config'
echo 'ok: host auth mounts are provider-scoped'

stage_provider() {
  local agent="$1" staged="$2" expected="$3"
  # Consumed by stage_host_auth from the sourced helper.
  # shellcheck disable=SC2034
  AGENT="${agent}"
  STATE_DIR="${TMP}/state-${agent}"
  mkdir -p \
    "${STATE_DIR}/home/.codex" \
    "${STATE_DIR}/home/.claude" \
    "${STATE_DIR}/home/.config/opencode" \
    "${STATE_DIR}/home/.local/share/opencode"
  stage_host_auth
  [[ -L "${staged}" ]] || fail "${agent} native agents directory was not staged"
  [[ "$(readlink "${staged}")" == "${expected}" ]] \
    || fail "${agent} native agents directory has the wrong target"
}

security() { return 1; }
stage_provider codex "${TMP}/state-codex/home/.codex/agents" /host-auth/codex/agents
stage_provider claude "${TMP}/state-claude/home/.claude/agents" /host-auth/claude/agents
stage_provider opencode "${TMP}/state-opencode/home/.config/opencode/agents" \
  /host-auth/opencode-config/agents
echo 'ok: native provider agent profiles are staged'

CONFIG_HOME="${TMP}/mosaico"
mkdir -p "${CONFIG_HOME}/agents"
printf '%s\n' \
  '{"codex-app-server":{"harness":"codex","transport":"app-server","args":["-c","model=test"]}}' \
  >"${CONFIG_HOME}/harnesses.json"
printf '%s\n' \
  '{"slug":"codex","created_at":1,"perSessionKey":true,"harness":"codex-app-server"}' \
  >"${CONFIG_HOME}/agents/codex.json"
MOSAICO_AGENT=codex MOSAICO_HOME="${CONFIG_HOME}" MOSAICO_DOCTOR_CONFIG_ONLY=1 \
  bash "${ROOT}/containers/mosaico/doctor" codex 1 >/dev/null
echo 'ok: doctor accepts current harness and keyless agent schemas'

printf '%s\n' \
  '{"codex-app-server":{"harness":"codex","transport":"app-server","profile":{}}}' \
  >"${CONFIG_HOME}/harnesses.json"
if MOSAICO_AGENT=codex MOSAICO_HOME="${CONFIG_HOME}" MOSAICO_DOCTOR_CONFIG_ONLY=1 \
  bash "${ROOT}/containers/mosaico/doctor" codex 1 >/dev/null 2>&1; then
  fail 'doctor accepted removed harness profile configuration'
fi

printf '%s\n' \
  '{"codex-app-server":{"harness":"codex","transport":"app-server"}}' \
  >"${CONFIG_HOME}/harnesses.json"
printf '%s\n' \
  '{"slug":"codex","created_at":1,"perSessionKey":true,"harness":"codex-app-server","secret_key":"stale","public_key":"stale"}' \
  >"${CONFIG_HOME}/agents/codex.json"
if MOSAICO_AGENT=codex MOSAICO_HOME="${CONFIG_HOME}" MOSAICO_DOCTOR_CONFIG_ONLY=1 \
  bash "${ROOT}/containers/mosaico/doctor" codex 1 >/dev/null 2>&1; then
  fail 'doctor accepted persisted keys for a per-session agent'
fi
echo 'ok: doctor rejects removed harness fields and redundant per-session keys'

mkdir -p "${TMP}/fake-bin"
# These are literal lines for the fake executable, not expressions here.
# shellcheck disable=SC2016
printf '%s\n' \
  '#!/bin/sh' \
  'if [ "${1:-} ${2:-}" = "system status" ]; then exit 0; fi' \
  'if [ "${1:-} ${2:-}" = "image inspect" ]; then exit 0; fi' \
  'if [ "${1:-}" = "run" ]; then exec /bin/sleep "${FAKE_CONTAINER_SLEEP:-0}"; fi' \
  'exit 2' >"${TMP}/fake-bin/container"
chmod +x "${TMP}/fake-bin/container"
LOCK_STATE="${TMP}/lock-state"
PATH="${TMP}/fake-bin:${PATH}" \
  MOSAICO_CONTAINER_HOST_AUTH=0 \
  MOSAICO_CONTAINER_STATE="${LOCK_STATE}" \
  FAKE_CONTAINER_SLEEP=0.5 \
  bash "${ROOT}/containers/mosaico/run" --profile opencode shell &
FIRST_RUNNER=$!
for _ in {1..50}; do
  [[ -d "${LOCK_STATE}.container-lock" ]] && break
  sleep 0.01
done
[[ -d "${LOCK_STATE}.container-lock" ]] || fail 'first runner did not take profile lock'
if LOCK_OUTPUT="$(
  PATH="${TMP}/fake-bin:${PATH}" \
    MOSAICO_CONTAINER_HOST_AUTH=0 \
    MOSAICO_CONTAINER_STATE="${LOCK_STATE}" \
    bash "${ROOT}/containers/mosaico/run" --profile opencode shell 2>&1
)"; then
  fail 'second same-profile runner bypassed the profile lock'
fi
grep -Fq 'container profile is already in use' <<<"${LOCK_OUTPUT}" \
  || fail 'same-profile lock failure was not actionable'
wait "${FIRST_RUNNER}"
[[ ! -e "${LOCK_STATE}.container-lock" ]] \
  || fail 'profile lock was not released after the runner exited'
echo 'ok: same-profile commands cannot replace a live daemon socket'

bash -n \
  "${ROOT}/containers/mosaico/doctor" \
  "${ROOT}/containers/mosaico/host-auth.bash" \
  "${ROOT}/containers/mosaico/run"
echo 'ok: container helper scripts parse as bash'
