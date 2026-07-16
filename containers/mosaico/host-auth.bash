# shellcheck shell=bash

declare -a HOST_AUTH_MOUNTS=()

add_required_auth_dir_mount() {
  local source="$1" target="$2"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -d "${source}" ]]; then
    echo "required host auth/config directory missing: ${source}" >&2
    echo "set MOSAICO_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
    exit 1
  fi
  HOST_AUTH_MOUNTS+=(--mount "type=bind,source=${source},target=${target},readonly")
}

stage_auth_symlink() {
  local source="$1" target="$2" state_path="$3" required="${4:-required}"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -e "${source}" ]]; then
    if [[ "${required}" == "required" ]]; then
      echo "required host auth/config path missing: ${source}" >&2
      echo "set MOSAICO_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
      exit 1
    fi
    return 0
  fi
  if [[ -L "${state_path}" || -f "${state_path}" ]]; then
    rm -f "${state_path}"
  elif [[ -e "${state_path}" ]]; then
    echo "refusing to replace non-file auth path: ${state_path}" >&2
    exit 1
  fi
  ln -s "${target}" "${state_path}"
}

stage_auth_copy() {
  local source="$1" target="$2"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -f "${source}" ]]; then
    echo "required host auth/config path missing: ${source}" >&2
    echo "set MOSAICO_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
    exit 1
  fi
  rm -f "${target}"
  cp "${source}" "${target}"
  chmod u+w "${target}"
}

stage_auth_dir_copy() {
  local source="$1" target="$2" required="${3:-required}"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -d "${source}" ]]; then
    if [[ "${required}" == "required" ]]; then
      echo "required host auth/config directory missing: ${source}" >&2
      echo "set MOSAICO_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
      exit 1
    fi
    mkdir -p "${target}"
    return 0
  fi
  if [[ -L "${target}" || -f "${target}" ]]; then
    rm -f "${target}"
  elif [[ -e "${target}" ]]; then
    chmod -R u+w "${target}" 2>/dev/null || true
    rm -rf "${target}"
  fi
  cp -a "${source}" "${target}"
  chmod -R u+w "${target}"
}

stage_claude_credentials() {
  local target="$1" tmp="${1}.tmp"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  rm -f "${tmp}"
  if command -v security >/dev/null 2>&1 \
    && command -v jq >/dev/null 2>&1 \
    && security find-generic-password -l "Claude Code-credentials" -w >"${tmp}" 2>/dev/null \
    && jq -e '.claudeAiOauth.accessToken and .claudeAiOauth.refreshToken' "${tmp}" >/dev/null 2>&1; then
    mv "${tmp}" "${target}"
    chmod 600 "${target}"
    return 0
  fi
  rm -f "${tmp}"
  stage_auth_copy "${HOST_HOME}/.claude/.credentials.json" "${target}"
}

stage_claude_settings() {
  local source="$1" target="$2" required="${3:-required}" tmp="${2}.tmp"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -f "${source}" ]]; then
    if [[ "${required}" == "optional" ]]; then
      rm -f "${target}"
      return 0
    fi
    echo "required host auth/config path missing: ${source}" >&2
    echo "set MOSAICO_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
    exit 1
  fi
  rm -f "${tmp}"
  if ! command -v jq >/dev/null 2>&1; then
    echo "required command missing for Claude settings staging: jq" >&2
    exit 1
  fi
  if jq '
      del(.hooks, .statusLine)
      | .env = ((.env // {}) + {
          MOSAICO_CONFIG: "/state/mosaico/config.json",
          MOSAICO_HOME: "/state/mosaico",
          MOSAICO_BIN: "/state/target/debug/mosaico"
        })
    ' "${source}" >"${tmp}" 2>/dev/null; then
    mv "${tmp}" "${target}"
  else
    rm -f "${tmp}"
    echo "failed to parse Claude settings JSON: ${source}" >&2
    exit 1
  fi
  chmod u+w "${target}"
}

stage_codex_named_profiles() {
  local source target name
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  for target in "${STATE_DIR}/home/.codex"/*.config.toml; do
    [[ -L "${target}" ]] || continue
    [[ "$(readlink "${target}")" == /host-auth/codex/* ]] || continue
    rm -f "${target}"
  done
  for source in "${HOST_HOME}/.codex"/*.config.toml; do
    [[ -e "${source}" ]] || continue
    name="${source##*/}"
    stage_auth_symlink "${source}" "/host-auth/codex/${name}" \
      "${STATE_DIR}/home/.codex/${name}"
  done
}

build_host_auth_mounts() {
  HOST_AUTH_MOUNTS=()
  add_required_auth_dir_mount "${HOST_HOME}/.mosaico" "/host-auth/mosaico"
  add_required_auth_dir_mount "${HOST_HOME}/.codex" "/host-auth/codex"
  add_required_auth_dir_mount "${HOST_HOME}/.claude" "/host-auth/claude"
  add_required_auth_dir_mount "${HOST_HOME}/.local/share/opencode" "/host-auth/opencode-data"
  add_required_auth_dir_mount "${HOST_HOME}/.config/opencode" "/host-auth/opencode-config"
}

stage_host_auth() {
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi

  stage_auth_symlink "${HOST_HOME}/.codex/auth.json" \
    "/host-auth/codex/auth.json" "${STATE_DIR}/home/.codex/auth.json"
  stage_auth_symlink "${HOST_HOME}/.codex/config.toml" \
    "/host-auth/codex/config.toml" "${STATE_DIR}/home/.codex/config.toml"
  stage_auth_symlink "${HOST_HOME}/.codex/config.json" \
    "/host-auth/codex/config.json" "${STATE_DIR}/home/.codex/config.json" optional
  stage_codex_named_profiles

  stage_auth_copy "${HOST_HOME}/.claude.json" "${STATE_DIR}/home/.claude.json"
  stage_auth_copy "${HOST_HOME}/.claude.json" "${STATE_DIR}/home/.claude/.claude.json"
  stage_claude_credentials "${STATE_DIR}/home/.claude/.credentials.json"
  stage_claude_settings "${HOST_HOME}/.claude/settings.json" "${STATE_DIR}/home/.claude/settings.json"
  stage_claude_settings "${HOST_HOME}/.claude/settings.local.json" \
    "${STATE_DIR}/home/.claude/settings.local.json" optional
  stage_auth_symlink "${HOST_HOME}/.claude/.env" \
    "/host-auth/claude/.env" "${STATE_DIR}/home/.claude/.env" optional
  stage_auth_symlink "${HOST_HOME}/.claude/agents" \
    "/host-auth/claude/agents" "${STATE_DIR}/home/.claude/agents" optional
  stage_auth_dir_copy "${HOST_HOME}/.claude/skills" "${STATE_DIR}/home/.claude/skills" optional
  stage_auth_symlink "${HOST_HOME}/.claude/custom-skills" \
    "/host-auth/claude/custom-skills" "${STATE_DIR}/home/.claude/custom-skills" optional
  stage_auth_symlink "${HOST_HOME}/.claude/commands" \
    "/host-auth/claude/commands" "${STATE_DIR}/home/.claude/commands" optional

  stage_auth_symlink "${HOST_HOME}/.local/share/opencode/auth.json" \
    "/host-auth/opencode-data/auth.json" "${STATE_DIR}/home/.local/share/opencode/auth.json"
  stage_auth_symlink "${HOST_HOME}/.local/share/opencode/account.json" \
    "/host-auth/opencode-data/account.json" "${STATE_DIR}/home/.local/share/opencode/account.json"
  stage_auth_symlink "${HOST_HOME}/.config/opencode/opencode.jsonc" \
    "/host-auth/opencode-config/opencode.jsonc" "${STATE_DIR}/home/.config/opencode/opencode.jsonc"
}
