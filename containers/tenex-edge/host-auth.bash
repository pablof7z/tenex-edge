# shellcheck shell=bash

declare -a HOST_AUTH_MOUNTS=()

add_required_auth_dir_mount() {
  local source="$1" target="$2"
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi
  if [[ ! -d "${source}" ]]; then
    echo "required host auth/config directory missing: ${source}" >&2
    echo "set TENEX_EDGE_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
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
      echo "set TENEX_EDGE_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
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
    echo "set TENEX_EDGE_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
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
      echo "set TENEX_EDGE_CONTAINER_HOST_AUTH=0 only for non-agent smoke tests" >&2
      exit 1
    fi
    mkdir -p "${target}"
    return 0
  fi
  if [[ -L "${target}" || -f "${target}" ]]; then
    rm -f "${target}"
  elif [[ -e "${target}" ]]; then
    rm -rf "${target}"
  fi
  cp -a "${source}" "${target}"
  chmod -R u+w "${target}"
}

build_host_auth_mounts() {
  HOST_AUTH_MOUNTS=()
  add_required_auth_dir_mount "${HOST_HOME}/.tenex-edge" "/host-auth/tenex-edge"
  add_required_auth_dir_mount "${HOST_HOME}/.codex" "/host-auth/codex"
  add_required_auth_dir_mount "${HOST_HOME}/.claude" "/host-auth/claude"
  add_required_auth_dir_mount "${HOST_HOME}/.local/share/opencode" "/host-auth/opencode-data"
  add_required_auth_dir_mount "${HOST_HOME}/.config/opencode" "/host-auth/opencode-config"
}

stage_host_auth() {
  if [[ "${HOST_AUTH}" != "1" ]]; then
    return 0
  fi

  stage_auth_symlink "${HOST_HOME}/.tenex-edge/providers.json" \
    "/host-auth/tenex-edge/providers.json" "${STATE_DIR}/tenex/edge/providers.json"
  stage_auth_symlink "${HOST_HOME}/.tenex-edge/llms.json" \
    "/host-auth/tenex-edge/llms.json" "${STATE_DIR}/tenex/edge/llms.json"

  stage_auth_symlink "${HOST_HOME}/.codex/auth.json" \
    "/host-auth/codex/auth.json" "${STATE_DIR}/home/.codex/auth.json"
  stage_auth_symlink "${HOST_HOME}/.codex/config.toml" \
    "/host-auth/codex/config.toml" "${STATE_DIR}/home/.codex/config.toml"
  stage_auth_symlink "${HOST_HOME}/.codex/config.json" \
    "/host-auth/codex/config.json" "${STATE_DIR}/home/.codex/config.json" optional

  stage_auth_symlink "${HOST_HOME}/.claude/.credentials.json" \
    "/host-auth/claude/.credentials.json" "${STATE_DIR}/home/.claude/.credentials.json"
  stage_auth_copy "${HOST_HOME}/.claude/settings.json" "${STATE_DIR}/home/.claude/settings.json"
  stage_auth_symlink "${HOST_HOME}/.claude/settings.local.json" \
    "/host-auth/claude/settings.local.json" "${STATE_DIR}/home/.claude/settings.local.json" optional
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
